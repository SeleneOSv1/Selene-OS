# Selene Accounts Receivable Addendum — Customer Types + Credit Control + Payment Matching + Collections + Delivery/POS/Customer Relationship Boundary

```text
DOCUMENT TYPE:
MASTER DESIGN ADDENDUM / ACCOUNTS RECEIVABLE + CUSTOMER CREDIT + CASH APPLICATION + COLLECTIONS + CUSTOMER RELATIONSHIP BOUNDARY

STATUS:
MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

APPLIES TO:
Document 4 — Selene Accounts Receivable + Invoices + Debtor Chasing + Collections Master Design

PURPOSE:
Strengthen Document 4 with autonomous customer handling, Selene-connected vs non-Selene customer flows, credit qualification, credit limit control, debtor behavior tracking, bank-payment matching, chargebacks, refunds, credit notes, collections escalation, late interest/costs, goods delivery boundary, POS boundary, and future Customer Relationship Intelligence ownership.
```

## 0. Authority and Scope

AGENTS.md controls execution.

This is a docs-only master design addendum.

No runtime code was changed.

This document does not authorize implementation.

This document is Finance/Accounting Design Batch 2. It references future standalone engines but does not create them:

```text
PH1.LOGISTICS — Physical Delivery + Dispatch + Proof of Delivery + Customer Resolution Engine
PH1.POS.COMMERCE — POS + Retail + B2B + E-Commerce Commerce Execution Engine
PH1.CUSTOMER — Customer Relationship Intelligence + Customer Memory + Customer Experience Engine
PH1.CREDIT — Trade Customer Credit + Terms + Receivables Discipline Engine
PH1.AR.COLLECT — Debtors Collection + Legal Recovery + Insolvency Engine
```

Current repo truth does not prove runtime AR, Customer Relationship, Credit, Collections, Logistics, POS/Commerce, Banking, Payment Provider, or payment matching engines. These are future design boundaries pending Grand Architecture Reconciliation.

## 1. Master Addendum Law

Accounts Receivable is not just "send invoice and wait."

Selene must understand the customer, track the money, chase what is owed, match payments, protect the company from weak debtors, and communicate like a capable human.

The upgraded AR law is:

```text
Every customer has a relationship profile, credit profile, payment behavior history, delivery history, dispute history, and AR balance.

Every invoice must have a delivery path, payment path, collection path, and reconciliation path.

Every payment received must be matched to the correct customer and invoice before the invoice is marked paid.

Every customer credit decision must consider history, behavior, risk, outstanding balance, profitability, and current company policy.

Every debtor chasing action must be professional, human, persistent, lawful, and evidence-backed.

Every refund or credit note must follow approval and audit rules.

Every goods delivery event must feed AR and customer status, but physical delivery belongs to Logistics, not AR.

Every POS sale must feed AR/accounting where relevant, but POS belongs to Commerce, not AR.

Customer relationship intelligence must become its own Selene engine, not be buried inside AR.
```

AR owns receivables.

Customer Relationship owns the relationship.

Credit owns credit decisions.

Banking/Reconciliation owns payment proof.

Logistics owns goods delivery.

POS/Commerce owns sale execution.

Accounting owns journals.

Each engine gets a clear job instead of becoming an unsafe all-purpose financial owner.

## 2. Two Customer Worlds: Selene Customers vs Non-Selene Customers

Selene must support two customer types.

```text
1. Selene-connected customer
2. Non-Selene customer
```

They both owe money the same way, but the interaction channel is different.

## 2.1 Selene-Connected Customers

A Selene-connected customer has their own Selene system or approved Selene account.

Selene-to-Selene customers can exchange:

```text
invoice packet
bill packet
purchase order match
delivery status
payment scheduled status
payment sent status
payment confirmed status
dispute reason
credit note request
payment plan request
remittance advice
statement
account balance confirmation
```

Flow:

```text
Seller Selene issues invoice.
Buyer Selene receives invoice as AP candidate.
Buyer Selene validates PO / goods / service / tax.
Buyer Selene accepts, disputes, or requests correction.
Buyer Selene schedules payment.
Seller Selene tracks payment status automatically.
Banking confirms payment.
Both sides reconcile.
```

Benefits:

```text
fewer lost invoices
cleaner disputes
faster payment status
automatic remittance
better reconciliation
less human chasing
```

Selene-to-Selene does not remove approval.

Buyer company's Access/AP/Finance rules still apply.

## 2.2 Non-Selene Customers

Non-Selene customers use normal channels:

```text
email
SMS
WhatsApp
PDF
payment link
customer portal
phone follow-up
postal/mail if still used by the customer
manual remittance advice
bank transfer reference
```

Selene must handle them too.

Flow:

```text
AR creates invoice.
PH1.WRITE writes human message.
PH1.BCAST / PH1.DELIVERY sends through approved channel.
PH1.REM schedules reminders.
Banking checks payment.
AR matches payment.
If customer says "we paid," Selene searches bank/payment records.
If not found, Selene asks for remittance/proof.
If overdue, collections flow begins.
```

Rule:

```text
If a customer claims they did not receive the invoice, Selene can resend instantly by SMS, email, WhatsApp, portal link, or another approved channel and record delivery proof.
```

Example:

```text
Customer: We never got the invoice.
Selene: I've resent it to your email and SMS with a secure payment link. I'll also record this delivery confirmation on your account.
```

## 3. Customer Qualification And Tracking

Selene must qualify customers before giving credit and keep monitoring them after approval.

This belongs mainly to a future standalone engine:

```text
PH1.CREDIT
Trade Customer Credit + Terms + Receivables Discipline Engine
```

This is a future design target only. It is not implemented by this addendum.

AR must consume and respect its decisions.

### Customer qualification fields

```text
customer_id
customer_type
legal_name
trading_name
country
region
industry
business_age
tax_registration
company_registration
credit_application_ref
trade_references
bank_reference
external_credit_score_ref
internal_payment_history
proposed_credit_limit
approved_credit_limit
payment_terms
credit_status
risk_status
approved_by
audit_ref
```

### Customer tracking fields

```text
average_days_to_pay
days_sales_outstanding
late_payment_count
broken_promise_count
dispute_count
chargeback_count
refund_count
credit_note_count
over_limit_count
credit_hold_count
writeoff_history
legal_history
insolvency_warning
profitability_score
relationship_score
account_manager_id
last_reviewed_at
next_review_at
```

Selene must not treat customers as static. A good customer can weaken. A weak customer can recover. A bad actor can look pleasant and still remain high risk. History stays.

## 4. Customer Credit Personality: Good, Disorganized, Shaky, Bad Actor

Not all late customers are the same.

Selene must distinguish behavior patterns.

### 4.1 Good customer

```text
Pays on time or close to terms.
Low disputes.
Good profitability.
Strong relationship.
```

Possible actions:

```text
increase credit limit
offer better terms
prioritize service
allow temporary over-limit approval
offer early payment discount only if useful
```

### 4.2 Disorganized but reliable customer

```text
Often pays late.
Usually pays when reminded.
Few broken promises after direct follow-up.
Low bad-debt risk.
May have messy internal process.
```

Possible actions:

```text
increase credit cautiously if profitable
send earlier reminders
use SMS/WhatsApp in addition to email
offer autopay or Selene-to-Selene connection
request accounts payable contact
use payment links
shorten reminder timing
```

Selene note:

```text
Disorganized is not the same as dangerous.
```

### 4.3 Shaky customer

```text
Pays late repeatedly.
Breaks promises.
Disputes often.
Credit limit is regularly exceeded.
Risk increasing.
```

Actions:

```text
hold new orders
request partial payment
reduce credit limit
move to shorter terms
require deposit
move to cash-before-delivery
route account manager review
stop further credit until balance reduced
```

### 4.4 Bad actor / high-risk customer

```text
History of non-payment, chargebacks, fake disputes, insolvency, legal recovery, write-offs, or repeated broken promises.
```

Actions:

```text
cash-only
account suspended
no shipment without cleared funds
no credit increase
legal/collections review
senior approval for any exception
risk flag retained permanently unless formally overridden
```

Rule:

```text
Bad actor records, write-offs, legal history, insolvency events, chargebacks, and repeated broken promises must be retained and used in future credit evaluations.
```

No fresh start just because someone changed an email address.

## 5. Credit Limit Handling

If a customer exceeds or approaches their credit limit, Selene must act intelligently.

### Credit limit statuses

```text
within_limit
near_limit
at_limit
over_limit
temporary_over_limit_approved
credit_hold
cash_only
suspended
collections
restoration_review
```

### Standard flow

```text
customer order/invoice would exceed credit limit
-> PH1.CREDIT checks customer history
-> AR checks outstanding balance
-> Cashflow checks impact
-> Sales/Order engine checks business importance
-> Selene chooses path:
   approve within tolerance
   request payment
   request partial payment
   hold order
   ask credit manager
   reduce/restore/raise limit
   move to cash-only
```

### Good customer over limit

```text
Customer has excellent payment history.
Order would exceed limit by small amount.
Selene may recommend temporary limit increase or manager approval.
```

Selene says:

```text
This customer is slightly over their limit, but their payment history is strong. I recommend a temporary credit extension or asking them to pay the oldest invoice first.
```

### Shaky customer over limit

```text
Customer is late, disputes often, or has broken payment promises.
Selene should not increase limit automatically.
```

Selene says:

```text
This customer is over their limit and has a weak payment pattern. I recommend holding new orders until they pay down the balance or switch this order to upfront payment.
```

## 6. Cash-Only, Temporary Suspension, And Restoration

Selene must support reverting customers to cash payment or suspending credit.

### Status changes

```text
Normal -> Watch
Watch -> CreditHold
CreditHold -> CashOnly
CashOnly -> Suspended
Suspended -> Collections
CashOnly -> RestorationReview
RestorationReview -> Normal
```

### Cash-only triggers

```text
exceeds credit limit repeatedly
misses payment plan
chargeback received
broken promise to pay
90+ days overdue
insolvency warning
high-risk account behavior
management decision
```

### Restoration criteria

```text
balance paid down
payment plan completed
no new disputes
no recent chargebacks
management approval
new credit limit agreed
shorter terms accepted
monitoring period completed
```

Selene must monitor restored customers carefully.

```text
A restored customer is not automatically a normal customer. Selene must monitor them under a watch period.
```

## 7. Late Interest, Fees, Legal Costs, And Collection Costs

Selene must support contractual late charges.

These rules belong to PH1.CREDIT / PH1.AR.COLLECT / Tax-Legal policy, but AR must track the amounts.

### Late charge policy fields

```text
customer_id
contract_ref
grace_period_days
interest_start_day
interest_rate
interest_compounding_rule
admin_fee
legal_cost_recovery_allowed
collection_cost_recovery_allowed
maximum_charge_limit
country_law_ref
approval_required
audit_ref
```

### Flow

```text
invoice overdue
-> grace period checked
-> interest start date reached
-> interest calculated if lawful
-> customer notified
-> AR balance updated or memoed based on policy
-> Accounting/Tax treatment checked
-> audit
```

Example:

```text
Invoice overdue by 45 days.
Grace period: 14 days.
Interest begins after day 30.
Selene calculates interest from day 31 and adds it if contract and jurisdiction allow.
```

Selene wording:

```text
This invoice is now outside the grace period. Under the account terms, late interest may apply from 30 days overdue.
```

Rule:

```text
No late interest, legal cost, or collection fee may be added unless customer terms and jurisdiction rules allow it.
```

## 8. Debt Collection And Legal Recovery Boundary

Document 4 includes basic debtor chasing.

But serious debt management deserves a standalone future module:

```text
PH1.AR.COLLECT
Debtors Collection + Legal Recovery + Insolvency Engine
```

This is a future design target only. It is not implemented by this addendum.

PH1.AR.COLLECT should own:

```text
collections case lifecycle
formal demand process
legal notice candidate
insolvency warning
bad debt review
payment plan default
collections agency handoff
lawyer handoff
liquidator/administrator claim
write-off recommendation
recovery tracking
```

AR owns invoice balance.

PH1.AR.COLLECT owns collections/legal recovery workflow.

Accounting owns write-off posting.

Access/Authority approves formal escalation.

PH1.WRITE writes customer-safe/legal-safe wording.

GPT-5.5 may draft, never decide legal action.

## 9. Customer Says "I Paid" — Selene Payment Search Capability

This is a special Selene process.

If customer says:

```text
I already paid.
```

Selene must not argue. Selene searches.

### Search sources

```text
bank feeds
payment provider records
card settlement
payment link status
bank deposit descriptions
remittance advice
Selene-to-Selene payment confirmation
customer account history
open receipt queue
unmatched receipts
same amount receipts
similar reference receipts
same payer bank account
```

### Flow

```text
customer claims paid
-> Selene asks for payment date / amount / reference if needed
-> Banking/Reconciliation searches bank/payment records
-> AR searches unmatched receipts
-> candidate matches ranked
-> if confident match found, allocate receipt
-> if multiple candidates, ask customer for remittance or request review
-> if no match, politely explain no payment found yet
```

Selene says:

```text
Thanks — I'll check the bank and payment records now. If you have the payment reference, send it through and I'll match it faster.
```

If found:

```text
I found the payment. It came through yesterday with reference INV-1024. I've matched it to your invoice and marked it paid.
```

If not found:

```text
I can't see that payment in the bank or payment records yet. Please send the remittance or payment reference, and I'll keep checking.
```

## 10. Direct Bank Deposits And Ambiguous Matches

Customers may pay directly into the company bank account.

Sometimes two customers pay the same amount. Selene must not guess.

### Matching evidence

```text
invoice number in reference
customer name in payer field
payer bank account
payment amount
payment date
invoice amount
open balance
payment plan schedule
remittance advice
customer payment pattern
Selene-to-Selene confirmation
```

### Match confidence

```text
high_confidence
medium_confidence
low_confidence
ambiguous
no_match
```

### Ambiguous match rule

```text
If more than one customer/invoice could match, Selene must not mark invoice paid automatically.
```

Selene creates:

```text
UnmatchedReceiptReviewPacket:
  receipt_id
  amount
  received_at
  payer_name
  reference_text
  candidate_customer_ids
  candidate_invoice_ids
  confidence_scores
  required_action
  audit_ref
```

Selene says internally:

```text
This $5,000 receipt could match two open invoices. I need remittance evidence or review before allocation.
```

## 11. Chargebacks, Reversals, And Card Disputes

A card payment can look paid, then later reverse.

Selene must handle this.

### Chargeback sources

```text
customer card dispute
payment provider reversal
fraud claim
goods not received claim
duplicate charge claim
refund dispute
bank reversal
```

### Chargeback flow

```text
invoice marked paid by card
-> payment provider reports dispute/chargeback
-> Banking creates reversal evidence
-> AR reopens invoice exposure or creates dispute balance
-> Logistics/POS/source evidence checked
-> customer history updated
-> chargeback case opened
-> customer risk updated
-> Accounting posts reversal/fee
-> possible credit hold
```

### Chargeback fields

```text
chargeback_id
customer_id
invoice_id
payment_id
provider_ref
chargeback_reason
amount
fee_amount
evidence_deadline
source_evidence_refs
delivery_proof_refs
status
audit_ref
```

Statuses:

```text
Opened
EvidenceRequired
EvidenceSubmitted
Won
Lost
Reversed
FeeApplied
CustomerBalanceReopened
Closed
```

Selene says:

```text
This payment was reversed by the card provider. I've reopened the invoice balance and started a chargeback review using the delivery and sale evidence.
```

## 12. Bank And Payment Provider Fees

Selene must verify card fees, bank fees, and abnormal deductions.

Fee types:

```text
card processing fee
chargeback fee
refund fee
bank transfer fee
foreign exchange fee
settlement fee
merchant service fee
late banking fee
failed payment fee
```

Selene checks:

```text
provider fee schedule
expected fee
actual fee
variance
country/currency
transaction type
merchant rate
refund/chargeback reason
```

If abnormal:

```text
PaymentFeeExceptionPacket:
  fee_id
  payment_id
  expected_fee
  actual_fee
  variance
  provider_ref
  review_required
  audit_ref
```

Selene says:

```text
The card fee on this settlement is higher than expected. I've opened a review before posting the final fee allocation.
```

## 13. Refunds And Credit Notes Governance

Refunds and credit notes both return value to the customer, but they are not the same.

```text
Credit note = reduces what the customer owes or creates customer credit.
Refund = actual money leaves the company.
```

### Credit note approval

Credit note approval depends on:

```text
amount
reason
customer risk
tax impact
sales owner
goods returned
delivery evidence
dispute status
manager authority
finance authority
```

Small credit note may require one approver.

Large credit note may require two approvers.

### Refund approval

Refunds are payment actions and may require:

```text
refund authority
payment authority
cashflow check
original payment verification
customer bank/payment destination verification
step-up for approver
dual approval above threshold
audit
```

### Refund risk rules

```text
refund to original payment method preferred
refund to different account = high risk
large refund = dual approval
refund after chargeback = special review
refund for bad actor = collections/credit review
```

### Approval packet

```text
CustomerValueReturnApprovalPacket:
  request_id
  type: credit_note / refund
  customer_id
  invoice_id
  amount
  reason_code
  source_evidence_refs
  tax_impact_ref
  approval_required
  required_approvers
  step_up_required
  status
  audit_ref
```

Rule:

```text
Credit notes reduce receivable value.
Refunds move money.
Both require reason, authority, tax/accounting treatment, and audit.
```

## 14. Instant Invoice Resend And Multi-Channel Delivery

Selene must be able to resend invoices instantly.

Channels:

```text
SMS
email
WhatsApp
secure portal
payment link
PDF
Selene-to-Selene
```

Flow:

```text
customer says invoice not received
-> AR checks invoice status
-> PH1.WRITE prepares short message
-> BCAST/DELIVERY sends through approved channel
-> delivery receipt recorded
-> reminder schedule updated
```

Selene says:

```text
No problem — I've resent the invoice by SMS and email with a secure payment link. I'll record this delivery confirmation on the account.
```

AR must preserve:

```text
sent_at
channel
recipient
delivery_status
view_status if available
payment_link_status
audit_ref
```

## 15. Goods Delivery Boundary

Delivery of goods is not AR.

It needs a standalone future engine:

```text
PH1.LOGISTICS
Physical Delivery + Dispatch + Proof of Delivery + Customer Resolution Engine
```

This is a future design target only. It is not implemented by this addendum.

PH1.LOGISTICS owns:

```text
dispatch
pick/pack
carrier assignment
tracking
delivery route
proof of delivery
delivery photos
customer signature
delivery failure
short delivery
damaged delivery
wrong goods
return pickup
customer delivery complaint
delivery claim
```

AR consumes delivery evidence.

AR does not create delivery truth.

### AR delivery dependency

Some invoices may require delivery proof before:

```text
invoice issuance
revenue recognition
payment demand
collections escalation
chargeback defense
refund denial
```

Example:

```text
Customer says goods never arrived.
Selene checks Logistics proof of delivery.
If delivered with signature/photo, AR can continue collection or dispute response.
If no proof, AR pauses collection and opens delivery resolution.
```

## 16. POS / Commerce Boundary

POS is not AR.

It needs standalone future engine:

```text
PH1.POS.COMMERCE
POS + Retail + B2B + E-Commerce Commerce Execution Engine
```

This is a future design target only. It is not implemented by this addendum.

POS/Commerce owns:

```text
sale
checkout
receipt
discount
payment method
refund trigger
store credit
gift card
cash drawer
card terminal
product sold
customer sale evidence
```

AR consumes POS evidence when:

```text
trade customer buys on account
B2B invoice created from sale
payment fails after sale
card chargeback reopens debt
customer refund/credit note required
sale creates commission trigger
sale creates tax/revenue/inventory accounting events
```

POS must feed Accounting/POS revenue, Tax, Inventory/COGS, Banking, and AR where relevant.

AR does not own checkout.

## 17. Future Customer Relationship Intelligence Engine

Customer relationship is not AR.

It needs standalone future engine:

```text
PH1.CUSTOMER
Customer Relationship Intelligence + Customer Memory + Customer Experience Engine
```

This is a future design target only. It is not implemented by this addendum.

PH1.CUSTOMER should own:

```text
customer profile
contacts
relationship history
preferred channels
tone preferences
customer sentiment
complaints
loyalty
relationship strength
account manager context
customer promises
customer habits
special terms
customer emotional/communication style
customer opportunities
customer risk notes from approved evidence
customer memory governance
```

Selene should build deep, human-like relationships with customers.

But:

```text
Customer Relationship does not own invoices.
Customer Relationship does not approve credit.
Customer Relationship does not collect debt directly.
Customer Relationship does not decide accounting truth.
Customer Relationship provides context and communication intelligence.
```

Future standard:

```text
Selene should understand the customer well enough to communicate personally, remember preferences lawfully, know payment habits, recognize frustration, manage account relationships, and improve customer experience without manipulating or hiding financial truth.
```

Example:

```text
Customer always pays after SMS, not email.
Selene learns approved preference.
Future invoices/reminders use SMS first.
```

This is where emotional intelligence belongs: customer-aware communication, not unsafe emotional manipulation.

## 18. Customer Credit Engine Boundary

Credit belongs in standalone:

```text
PH1.CREDIT
Trade Customer Credit + Terms + Receivables Discipline Engine
```

This is a future design target only. It is not implemented by this addendum.

PH1.CREDIT owns:

```text
credit application
credit approval
credit limit
payment terms
risk rating
credit hold
cash-only status
restoration review
bad actor history
late payment behavior
customer credit policy
```

AR consumes credit decisions.

Sales/Logistics must check credit status before releasing account orders or goods where policy requires.

## 19. Human-Like Customer Communication Law

Selene must not sound like a dead billing robot.

Customer communication should be:

```text
clear
respectful
warm when appropriate
firm when necessary
specific
evidence-backed
short enough to read
channel-aware
relationship-aware
professionally persistent
```

Selene should adapt:

```text
good customer -> friendly and helpful
disorganized customer -> earlier reminders and simple payment links
shaky customer -> firm and structured
bad actor -> formal and controlled
disputed invoice -> calm and evidence-focused
high-value customer -> account-manager-aware
Selene-connected customer -> system-to-system clean updates
```

PH1.WRITE owns final wording.

PH1.D/GPT-5.5 may draft tone.

AR/Customer/Credit owners provide truth.

## 20. Additions To Document 4 "What Must Not Happen"

Add:

```text
no customer marked paid just because they claimed they paid
no ambiguous bank receipt auto-allocated
no two same-amount payments guessed without proof
no invoice reminder sent to wrong contact/channel
no customer harassed outside policy
no credit limit automatically increased for shaky customer
no bad actor history erased
no cash-only customer restored without review
no late interest added without contract/jurisdiction basis
no legal or collection cost added without authority
no chargeback ignored after invoice was marked paid
no abnormal card/bank fee silently posted
no refund treated as casual credit note
no credit note/refund approved without correct authority
no invoice resend outside approved delivery protocol
no AR owning physical goods delivery
no AR owning POS checkout
no AR pretending to own customer relationship intelligence
no GPT-5.5 final customer credit decision
no implementation from this addendum alone
```

## 21. Required Logical Packets

Future logical packets:

```text
CustomerTypePacket
SeleneConnectedCustomerPacket
NonSeleneCustomerContactPacket
CustomerQualificationPacket
CustomerPaymentBehaviorPacket
CustomerCreditStatusPacket
CreditLimitReviewPacket
CreditHoldPacket
CashOnlyStatusPacket
CustomerRestorationReviewPacket
BadActorHistoryPacket
LateInterestAccrualPacket
CollectionCostPacket
CustomerPaidClaimPacket
BankReceiptCandidateMatchPacket
UnmatchedReceiptReviewPacket
ChargebackPacket
PaymentFeeExceptionPacket
CustomerValueReturnApprovalPacket
InvoiceResendRequestPacket
CustomerDeliveryProofDependencyPacket
CustomerRelationshipContextPacket
CustomerCommunicationTonePacket
```

Codex must later map these to repo equivalents.

Do not claim they currently exist.

## 22. Example — Disorganized But Good Customer Over Limit

```text
Customer: Northside Retail
Credit limit: AUD 50,000
Outstanding: AUD 52,000
History: pays late, but always pays after SMS reminder
New order: AUD 4,000
```

Selene result:

```text
risk_class: disorganized_reliable
recommendation:
  - send SMS reminder for oldest invoice
  - request partial payment
  - allow temporary credit extension if manager approves
  - review whether credit limit should be raised
```

Selene says:

```text
Northside is slightly over limit, but their history is reliable once reminded. I recommend requesting payment on the oldest invoice and approving a temporary extension only if the account manager agrees.
```

## 23. Example — Shaky Customer Over Limit

```text
Customer: Westline Supplies
Credit limit: AUD 30,000
Outstanding: AUD 39,000
History: broken promises, two disputes, one chargeback
New order: AUD 8,000
```

Selene result:

```text
risk_class: shaky
recommendation:
  - hold new order
  - request payment to reduce balance
  - switch order to upfront payment
  - no credit limit increase
```

Selene says:

```text
Westline is over limit and their payment behavior is weak. I recommend holding the new order until they pay down the balance or agree to upfront payment.
```

## 24. Example — Customer Says Paid But Bank Match Is Ambiguous

```text
Customer says: We paid yesterday.

Bank feed:
AUD 5,000 received
Reference: Payment
Two open invoices:
Customer A: AUD 5,000
Customer B: AUD 5,000
```

Selene result:

```text
match_status: ambiguous
action:
  - ask for remittance
  - create unmatched receipt review
  - do not mark paid yet
```

Selene says:

```text
I found a payment for that amount, but it could match more than one invoice. Please send the remittance or payment reference so I can apply it correctly.
```

## 25. Example — Chargeback Reopens Paid Invoice

```text
Invoice INV-2048 paid by card.
Goods shipped.
Card provider reports chargeback.
```

Selene result:

```text
invoice_status: customer_balance_reopened_or_dispute_pending
chargeback_case: opened
delivery_proof_requested: true
customer_risk_updated: true
accounting_reversal_needed: true
```

Selene says:

```text
The card payment for INV-2048 was reversed by the provider. I've reopened the payment review, requested delivery proof, and started the chargeback response process.
```

## 26. Example — Refund Approval

```text
Customer overpaid by AUD 12,000.
Refund requested to original payment method.
Policy: refunds over AUD 10,000 require CFO + Finance Manager approval.
```

Selene result:

```text
refund_status: pending_dual_approval
required_approvers:
  - CFO
  - Finance Manager
step_up_required: true
payment_instruction_blocked_until_approved: true
```

Selene says:

```text
The refund is valid, but it is above the approval threshold. I've prepared the approval request for Finance Manager and CFO review.
```

## 27. Future Simulation Targets

Add these future simulations:

```text
SIM_AR_017_selene_connected_customer_invoice_exchange
SIM_AR_018_non_selene_customer_invoice_resend_by_sms_email_whatsapp
SIM_AR_019_customer_says_paid_bank_search_match_found
SIM_AR_020_customer_says_paid_bank_match_ambiguous
SIM_AR_021_direct_bank_payment_unmatched_receipt_review
SIM_AR_022_good_disorganized_customer_over_limit_temporary_extension
SIM_AR_023_shaky_customer_over_limit_order_hold
SIM_AR_024_bad_actor_customer_cash_only_status
SIM_AR_025_customer_restoration_after_cash_only_watch_period
SIM_AR_026_late_interest_after_grace_period
SIM_AR_027_chargeback_reopens_paid_invoice
SIM_AR_028_abnormal_card_fee_review
SIM_AR_029_credit_note_dual_approval
SIM_AR_030_refund_original_payment_method
SIM_AR_031_refund_to_different_account_high_risk
SIM_AR_032_delivery_proof_required_for_collection_response
SIM_AR_033_pos_sale_chargeback_ar_reopen
SIM_CUSTOMER_001_customer_relationship_context_used_for_tone
SIM_CREDIT_001_credit_limit_increase_good_customer
SIM_CREDIT_002_credit_limit_denied_shaky_customer
SIM_COLLECT_001_90_day_debt_escalation
```

## 28. Final Addendum Architecture Sentence

Selene Accounts Receivable must become a customer-aware, credit-aware, bank-aware, delivery-aware, and human-communicating receivables system: it must distinguish Selene-connected and non-Selene customers, qualify and track customer risk, manage credit limits intelligently, preserve bad-actor history, chase debtors professionally, apply late interest and costs only where lawful, match bank payments safely, handle ambiguous receipts, reopen invoices after chargebacks, govern refunds and credit notes through approval, resend invoices instantly through approved channels, consume delivery and POS evidence without owning those engines, and hand customer relationship depth to a dedicated Customer Relationship Intelligence engine.
