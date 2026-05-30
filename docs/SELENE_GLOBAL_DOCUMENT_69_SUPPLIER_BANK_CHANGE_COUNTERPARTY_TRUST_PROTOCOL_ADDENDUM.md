# Global Document 69 — Supplier Bank Change + Selene-to-Selene Counterparty Trust Protocol Addendum

```text id="doc69_status"
DOCUMENT TYPE:
GLOBAL MASTER ARCHITECTURE ADDENDUM / SUPPLIER TRUST PROTOCOL DESIGN

GLOBAL DOCUMENT NUMBER:
69

PARENT ENGINE:
Global Document 68 — Selene Supplier Intelligence Engine

ENGINE:
PH1.SUPPLIER.BANK_TRUST / PH1.COUNTERPARTY_TRUST / PH1.SELENE_TO_SELENE

FULL NAME:
Selene Supplier Bank Change, Counterparty Trust, Selene-to-Selene Supplier Messaging, Payment Safety, Fraud Prevention, and Supplier Identity Verification Protocol

STATUS:
FUTURE_CANONICAL_ARCHITECTURE
NOT_RUNTIME_IMPLEMENTATION
PENDING_REPO_TRUTH_ACTIVATION
PENDING_SIMULATION_MAPPING
CODEX_READY_MASTER_DESIGN
```

---

## 1. Purpose

This document defines how Selene handles supplier bank-detail changes, supplier payment identity, trusted counterparty messaging, and Selene-to-Selene supplier communication.

It exists because supplier bank changes are one of the most dangerous moments in AP and supplier payments.

The FBI’s IC3 has described Business Email Compromise as a global scam reported across all U.S. states and 186 countries, with fraudulent transfers received in more than 140 countries; its 2024 public service announcement described BEC as a “$55 billion scam.” ([Internet Crime Complaint Center][1])

So Selene must never behave like this:

```text id="fl4t05"
Supplier emailed new bank details.
Looks official.
Let’s update it.
Pay them.
Oops.
```

That is not automation. That is fraud with a button.

Selene’s model is:

```text id="h2rs7a"
Supplier initiates bank change.
Supplier authority is verified.
Buyer Selene validates request.
Buyer policy decides.
Payment remains protected.
Audit records everything.
```

Supplier bank changes must be treated as protected identity events, not normal supplier profile edits.

---

## 2. Core Selene Law

```text id="bank_core_law"
Supplier bank-detail changes must originate from the supplier side and must never be accepted from an unverified buyer-side edit, suspicious email, invoice note, or casual internal request.

Supplier Selene may request.
Buyer Selene validates.
Buyer policy decides.
Payment remains protected until trust, authority, evidence, and audit are satisfied.
```

Selene must reduce human work by:

```text id="y507r6"
detecting supplier bank changes automatically
verifying supplier identity automatically where possible
validating authority automatically where possible
matching supplier records
checking trust levels
checking fraud signals
checking AP holds
checking payment readiness
placing temporary payment holds
routing only true exceptions
communicating with supplier automatically
recording audit evidence
```

But Selene must never reduce safety by treating bank details as a simple editable field.

A changed supplier bank account is not “profile update.” It is “possible money teleportation to fraud goblin.” Tiny distinction, large consequences.

---

## 3. Parent Relationship to Supplier Intelligence

This document is an addendum to:

```text id="parent_supplier"
Global Document 68 — Selene Supplier Intelligence Engine
```

Document 68 owns:

```text id="doc68_owns"
supplier identity
supplier qualification
supplier risk
supplier obligations
supplier scorecard
supplier approval status
supplier payment safety signals
supplier bank safety state
```

Document 69 expands:

```text id="doc69_expands"
supplier-originated bank changes
Selene-to-Selene counterparty verification
trusted supplier messages
supplier payment identity protection
bank-change state machine
counterparty trust levels
fraud/risk controls
buyer-side validation
supplier portal rules
```

Document 69 does **not** replace Supplier Intelligence.

It gives Supplier Intelligence a spine, a lock, and a healthy distrust of PDFs.

---

## 4. Engine Boundary

### 4.1 PH1.SUPPLIER.BANK_TRUST owns

```text id="banktrust_owns"
supplier bank change protocol
supplier-originated bank change requests
supplier authority verification evidence
counterparty Selene identity validation
buyer-side supplier record matching
supplier bank safety state
bank-change trust scoring
bank-change payment hold recommendation
supplier payment identity risk signals
bank-change audit evidence
Selene-to-Selene supplier message validation
manual non-Selene supplier bank-change verification workflow
```

### 4.2 PH1.SUPPLIER.BANK_TRUST does not own

```text id="banktrust_not_own"
bank payment execution
final supplier invoice validation
ledger posting
supplier invoice matching
purchase order creation
receiving truth
tax treatment
legal contract final approval
actual bank account custody
```

### 4.3 Correct owner split

```text id="banktrust_owner_split"
Supplier Engine = supplier identity, risk, status, and bank safety signal.
Supplier Bank Trust Addendum = how supplier bank changes and counterparty trust are verified.
AP / Creditors = invoice payable truth.
Supplier Payment / Banking Handoff = payment scheduling and provider/bank handoff.
Banking / Payment Provider = actual money movement.
Cashflow = whether payment timing is safe.
Access / Authority = who may approve protected exceptions.
Audit = proof.
```

Supplier Bank Trust does not move money.

It protects the path before money moves.

---

## 5. Supplier Types

Selene must support two supplier types.

```text id="supplier_types_bank"
1. Selene-connected supplier
2. Non-Selene supplier
```

### 5.1 Selene-connected supplier

A supplier that operates its own Selene instance.

This enables authenticated machine-to-machine counterparty messages:

```text id="connected_messages"
bank change request
credit note issued
replacement goods sent
refund confirmed
invoice submitted
PO acknowledgement
shipment delay
supplier statement
certificate renewal
price list update
payment status query
```

### 5.2 Non-Selene supplier

A supplier that does not use Selene.

Uses verified manual channels:

```text id="non_connected_channels"
supplier portal
signed supplier document
verified supplier contact
callback to known phone number
secure upload
approved email plus independent verification
bank confirmation document
contract amendment
```

Selene should prefer Selene-connected suppliers because counterparty verification becomes stronger and workflows become easier.

But even Selene-connected messages are not blind commands.

They are authenticated requests.

Buyer Selene still validates and applies buyer-side policy.

---

## 6. Supplier-Originated Bank Change Rule

The safest rule:

```text id="supplier_originated_rule"
The supplier must request changes to their own payment bank details.

Buyer-side users may record, review, or approve a verified supplier-originated request, but should not casually create new supplier bank details from email, invoice text, or manual entry.
```

Bad flow:

```text id="bad_bank_flow"
Internal staff receives email.
Internal staff edits supplier bank.
Payment goes to new account.
Fraud happens.
Everyone discovers controls after the money leaves.
```

Selene flow:

```text id="selene_bank_flow"
Supplier requests change.
Supplier authority verified.
Request signed / evidenced.
Buyer Selene validates supplier identity.
Buyer policy checks risk.
Payment hold applied if needed.
Change becomes active only when safe.
Audit records all steps.
```

The ACFE has warned that poor bank account management and weak controls can create major fraud risk, and its fraud guidance stresses due diligence and ongoing vendor monitoring, including verification of banking-information changes. ([Association of Certified Fraud Examiners][2])

Selene must make this automatic, boring, and difficult to bypass.

Boring controls are how money survives.

---

## 7. Selene-to-Selene Bank Change Flow

Supplier user says to Supplier Selene:

> “Change our bank details for payments from Buyer ABC.”

Supplier Selene checks:

```text id="supplier_side_checks"
requesting user identity
requesting user role
authority to change bank details
step-up authentication
dual approval if supplier policy requires
supplier legal entity identity
new bank evidence
effective date
reason for change
fraud/risk signals
audit proof
```

If valid, Supplier Selene creates:

```text id="bank_change_request_packet_name"
SupplierBankChangeRequestPacket
```

Buyer Selene receives and checks:

```text id="buyer_side_checks"
known supplier record
supplier Selene instance identity
supplier legal entity match
counterparty trust level
request signature/hash
freshness / replay protection
old bank reference
new bank masked data
effective date
supplier authority proof
buyer policy thresholds
open AP/payment risks
existing payment batches
recent fraud flags
required approval route
payment hold requirement
audit proof
```

Buyer Selene outcome:

```text id="buyer_outcomes"
accept under policy
accept after authority approval
request more evidence
reject
place payment hold
keep old bank active until effective date
suspend payments pending verification
```

Selene says:

> “Supplier ABC requested a bank change through their Selene. Their authority is verified, but buyer policy requires payment hold until Finance confirms because there is an open payment batch.”

That is human-like, safe, and blessedly less stupid than invoice-email roulette.

---

## 8. Non-Selene Supplier Bank Change Flow

If the supplier does not use Selene, Selene must require independent verification.

Non-Selene request may arrive via:

```text id="manual_request_sources"
supplier portal
signed supplier letter
secure upload
verified email from approved contact
phone callback to known number
contract amendment
bank proof document
```

Selene checks:

```text id="manual_verification_checks"
request source
sender identity
known supplier contact match
approved phone/contact verification
document authenticity
old vs new bank comparison
supplier legal entity match
bank country risk
payment timing risk
open payment batches
invoice pressure / urgency language
email domain changes
similar-name fraud signals
```

Selene says:

> “This supplier is not Selene-connected. I need verification from the approved supplier contact before bank details can change.”

If request came via invoice text:

> “The invoice includes new bank details, but no verified supplier bank-change request exists. I will not update payment details from invoice text.”

Good robot. Suspicious robot. Money-saving robot.

---

## 9. Counterparty Trust Levels

Selene must classify supplier communication trust.

```text id="trust_levels"
Level 0 — Unknown counterparty
Level 1 — Known supplier, unverified manual channel
Level 2 — Known supplier contact verified manually
Level 3 — Supplier portal verified
Level 4 — Selene-connected supplier
Level 5 — Selene-connected supplier + verified authority + step-up authentication + trusted history
```

Trust levels affect:

```text id="trust_level_effects"
evidence required
review depth
payment hold rules
whether auto-accept is allowed
whether authority approval is required
whether messages can update workflow state
```

Trust level does **not** remove policy.

Even Level 5 cannot directly mutate buyer records without buyer validation.

Selene-to-Selene is trust-enhanced, not remote-control accounting. We are civilized, barely.

---

## 10. Bank Detail Data Protection

Supplier bank details are sensitive.

Selene must protect:

```text id="bank_data_protection"
full account numbers
routing details
IBAN/SWIFT
beneficiary name
bank address
payment rail data
supporting bank documents
change history
```

Rules:

```text id="bank_data_rules"
mask bank details by default
show full details only to authorized roles
never expose full details in general chat
audit every view/change
encrypt/store securely in future implementation
separate bank-reference ID from raw bank data
```

Selene should say:

> “Supplier bank details are verified and active. Full details are restricted.”

Not:

> “Here is the entire bank account in a conversation thread for vibes.”

No. Bad system. Sit.

---

## 11. Supplier Bank Safety States

```text id="bank_safety_states"
NotProvided
PendingVerification
Verified
ChangeRequested
SupplierAuthorityVerified
BuyerValidationPending
BuyerApprovalPending
ApprovedPendingEffectiveDate
RecentlyChanged
Active
Rejected
Suspended
PaymentHold
Archived
```

### State meaning

```text id="state_meaning"
NotProvided = no bank record available
PendingVerification = bank details exist but not verified
Verified = payment-safe under policy
ChangeRequested = supplier-originated change exists
SupplierAuthorityVerified = supplier-side authority proof exists
BuyerValidationPending = buyer-side validation not complete
BuyerApprovalPending = buyer policy requires approval
ApprovedPendingEffectiveDate = approved but not active yet
RecentlyChanged = recently activated and heightened monitoring applies
Active = current verified bank
Rejected = change rejected
Suspended = bank use suspended
PaymentHold = payments blocked/held
Archived = old bank record retired
```

Supplier Payment Engine must respect this state.

Payment to `PendingVerification`, `ChangeRequested`, `Suspended`, or `PaymentHold` must be blocked unless protected emergency authority explicitly permits.

---

## 12. SupplierBankChangeRequestPacket

Logical future packet.

```text id="bank_change_packet"
SupplierBankChangeRequestPacket:
- request_id
- supplier_id
- supplier_legal_name
- supplier_selene_instance_id
- buyer_company_id
- requesting_user_id
- requesting_user_role
- supplier_authority_proof_ref
- step_up_auth_ref
- supplier_internal_approval_ref
- old_bank_ref
- new_bank_ref_masked
- new_bank_country
- new_bank_currency
- beneficiary_name
- effective_date
- reason
- document_evidence_refs
- fraud_risk_signals
- timestamp
- signature_hash
- replay_protection_token
- audit_ref
```

No raw full bank details in general packet views.

Sensitive data must be tokenized/masked and role-controlled in future implementation.

---

## 13. Buyer Validation Packet

```text id="buyer_validation_packet"
SupplierBankChangeValidationPacket:
- request_id
- buyer_supplier_record_id
- supplier_match_status
- supplier_identity_match_score
- counterparty_trust_level
- signature_validation_status
- freshness_status
- replay_check_status
- buyer_policy_result
- AP_open_invoice_count
- open_payment_batch_refs
- supplier_risk_status
- bank_country_risk
- payment_hold_required
- authority_required
- validation_result
- recommended_action
- audit_ref
```

Validation outcomes:

```text id="validation_outcomes"
AutoAcceptUnderPolicy
AcceptAfterApproval
RequestMoreEvidence
Reject
SuspendSupplierPayments
EscalateFraudReview
```

---

## 14. Bank Change Decision Packet

```text id="decision_packet"
SupplierBankChangeDecisionPacket:
- request_id
- decision
- decision_owner
- decision_policy_ref
- effective_date
- payment_hold_status
- old_bank_status
- new_bank_status
- rejection_reason
- next_action
- supplier_notification_ref
- AP_notification_ref
- Payment_notification_ref
- audit_ref
```

Decisions:

```text id="bank_change_decisions"
Approved
ApprovedWithHold
Rejected
EvidenceRequested
Suspended
FraudReview
Cancelled
```

---

## 15. Payment Hold Rules

Bank changes must integrate with AP and Supplier Payment.

Payment hold triggers:

```text id="payment_hold_triggers"
bank change requested
bank change pending verification
bank recently changed
supplier authority not verified
buyer validation incomplete
supplier restricted/blocked
open fraud signal
invoice urgency pressure
changed bank details on invoice only
supplier email/domain mismatch
open payment batch using old bank
```

Payment hold outcomes:

```text id="hold_outcomes"
hold all supplier payments
hold only payments to new bank
pay only old verified bank until effective date
require dual authority
require manual callback
block until review
```

Selene says:

> “Supplier bank change is pending. I removed this supplier from today’s payment batch until verification completes.”

That sentence prevents money from visiting strangers.

---

## 16. Active Payment Batch Protection

If a bank change arrives while a payment batch is pending:

Selene must:

```text id="payment_batch_protection"
detect open payment batch
detect supplier included
check current bank ref
freeze supplier payment line if needed
notify Supplier Payment Engine
notify AP
request validation
prevent duplicate payment after change
```

Possible outcomes:

```text id="payment_batch_outcomes"
continue with old verified bank if policy allows
hold payment until new bank verified
cancel supplier line from batch
create new batch after approval
```

Selene says:

> “This supplier has a bank-change request and is included in a pending payment batch. I’ve held the payment line pending verification.”

No “oops, payment already went.” That is not a control. That is a crime scene recap.

---

## 17. Bank Change Effective Dates

Bank changes may have effective dates.

Selene supports:

```text id="effective_date_handling"
immediate effective date
future effective date
specific invoice effective date
specific contract effective date
supplier-wide effective date
entity-specific effective date
currency-specific effective date
```

Rules:

```text id="effective_date_rules"
old bank remains active until change effective date unless suspended
new bank cannot receive payment before effective date unless authority allows
payment scheduled across transition must be reviewed
remittance must show which bank ref was used
```

Selene says:

> “The new bank is approved from 1 July. Payments before then will continue to the existing verified bank unless policy changes.”

Good. Boring. Safe.

---

## 18. Fraud Signal Detection

Selene must identify bank-change fraud signals.

Signals:

```text id="fraud_signals"
new bank details sent only by email
urgent payment pressure
email domain mismatch
lookalike supplier name
bank country differs from supplier country
beneficiary name mismatch
request near large payment date
new contact requests change
supplier refuses callback
invoice contains new bank without bank-change request
multiple bank changes in short period
supplier recently added
supplier already has open dispute
payment request and bank change arrive together
```

Fraud states:

```text id="fraud_states"
NoSignal
LowSignal
MediumSignal
HighSignal
FraudReviewRequired
Blocked
```

Selene says:

> “This request has three fraud signals: new bank on invoice only, urgency language, and bank country mismatch. I’m blocking payment and escalating review.”

A little paranoia is cheaper than a wire recall.

---

## 19. Supplier-to-Buyer Message Types

Selene-to-Selene counterparty messages extend beyond bank changes.

Supported message types:

```text id="counterparty_message_types"
SupplierBankChangeRequest
POAcknowledgement
ShipmentConfirmation
DeliveryDelayNotice
CreditNoteIssued
RefundConfirmation
ReplacementShipmentNotice
SupplierInvoiceSubmitted
SupplierStatementSubmitted
CertificateRenewalUploaded
PriceListUpdate
CatalogUpdate
PaymentStatusQuery
DisputeResponse
CancellationAcknowledgement
```

Each message requires:

```text id="counterparty_message_requirements"
counterparty identity
message type
source Selene instance
timestamp
reference IDs
signature/hash
replay protection
permission scope
buyer-side validation
audit_ref
```

No supplier message directly mutates buyer truth unless a buyer-side source engine accepts it.

Example:

```text id="message_example"
Supplier Selene sends CreditNoteIssued.
Buyer Selene validates supplier identity.
AP matches credit note to obligation.
Supplier Engine updates obligation.
Accounting later posts if appropriate.
```

Machine-to-machine does not mean “machine-to-machine chaos.”

---

## 20. Counterparty Identity Registry

Selene must store trusted counterparty identities.

```text id="counterparty_registry"
counterparty_id
supplier_id
counterparty_legal_name
selene_instance_id
verified_domain
verified_public_key_ref
trust_level
allowed_message_types
last_verified_at
last_message_at
risk_status
revocation_status
audit_ref
```

Trust actions:

```text id="trust_actions"
establish trust
rotate keys
suspend trust
revoke trust
renew trust
downgrade trust
escalate review
```

Selene says:

> “Supplier Selene identity is verified, but their trust key rotated yesterday. I need validation before accepting protected bank messages.”

Security is annoying, yes. Losing money is more annoying.

---

## 21. Supplier Portal Bank Change Rules

If supplier uses portal, bank change requires:

```text id="portal_bank_rules"
authorized supplier user
step-up authentication
supplier-side approval if configured
document evidence
old/new bank comparison
buyer validation
payment hold evaluation
audit
```

Supplier portal must not allow:

```text id="portal_bank_never"
anonymous bank edits
unverified user edits
email-only bank edits
bank changes without audit
direct payment execution after change
unmasked bank details to unauthorized users
```

Selene says:

> “Your bank change request was received. It will remain pending until buyer validation completes.”

Good supplier experience, safe buyer control.

---

## 22. Non-Selene Callback Verification

For non-Selene suppliers, callback must use trusted stored contact data.

Callback rules:

```text id="callback_rules"
call known supplier number already on file
do not use phone number from change request
record callback date/time
record verifier
record supplier contact spoken to
record confirmation result
attach evidence
```

If no trusted contact exists:

```text id="no_trusted_contact"
request verified supplier portal onboarding
request signed document
route to supplier qualification review
payment hold remains
```

Selene says:

> “The request includes a phone number, but it is not previously verified. I will not use it for callback verification.”

Fraudsters also own phones. Stunning revelation.

---

## 23. Bank Change and Supplier Score

Bank-change behavior affects supplier risk.

Risk score increases when:

```text id="bank_score_increase"
frequent bank changes
unverified change attempts
supplier sends new bank via invoice
supplier refuses verification
bank country mismatch
beneficiary mismatch
urgent payment pressure
failed verification
```

Risk may decrease when:

```text id="bank_score_decrease"
Selene-connected verified request
successful authority proof
consistent supplier history
clean verification record
no disputes/open fraud signals
```

Selene must not punish genuine changes unfairly, but it must keep risk visible.

Supplier scorecard may show:

```text id="bank_scorecard"
Bank Safety: Verified
Last Bank Change: 18 months ago
Fraud Signals: None
Payment Status: Safe
```

or:

```text id="bank_scorecard_risk"
Bank Safety: ChangeUnderReview
Fraud Signals: High
Payment Status: Hold
```

---

## 24. Supplier Payment Status Query

Suppliers may ask:

> “When will we be paid?”

If supplier is Selene-connected, Supplier Selene can send a payment status query.

Buyer Selene responds only with permitted information:

```text id="payment_query_allowed"
invoice received
invoice matched
payment scheduled date
payment paid date
credit note pending
disputed amount
bank verification pending
```

Buyer Selene must not reveal:

```text id="payment_query_not_allowed"
internal cashflow weakness
internal approval politics
sensitive bank details
other supplier info
management commentary
restricted financial data
```

Example response:

> “Invoice INV-884 is scheduled for payment on Friday. The disputed damaged-goods amount remains on hold pending credit note.”

Professional, useful, and not leaking cashflow panic. Everybody wins, except gossip.

---

## 25. Supplier Early / Urgent Payment Request

Supplier may request urgent or early payment.

Supplier Bank Trust contributes risk context.

Payment decision belongs to Supplier Payment and Cashflow.

Supplier Bank Trust checks:

```text id="early_payment_bank_checks"
bank verified
bank recently changed
supplier request channel trusted
supplier identity verified
payment request linked to clean invoice
urgency language risk
open bank-change request
```

If bank risk exists:

```text id="early_payment_hold"
block early/urgent payment until verification
```

Selene says:

> “Supplier requested urgent payment, but bank details changed recently. I will not accelerate payment until verification completes.”

Urgent fraud is still fraud. Just faster.

---

## 26. Audit Requirements

Every bank-change event must be audit-complete.

Audit records:

```text id="bank_audit"
request source
supplier identity
requesting user
supplier authority proof
buyer validation steps
hash/signature result
evidence documents
old bank ref
new bank ref masked
fraud signals
trust level
payment hold decision
authority decision
effective date
notifications sent
state changes
timestamps
actors
audit_ref
```

Selene must answer:

> “Why did we change Supplier ABC’s bank?”

and produce:

```text id="bank_audit_answer"
who requested
who verified
what evidence existed
what policy applied
when change took effect
which payments were affected
who approved exception if any
```

No audit? No bank change.

Simple and beautiful. Like a locked door.

---

## 27. Automation and Exception-Only Review

Selene auto-handles:

```text id="bank_auto_handles"
detect bank-change request
classify supplier connected/non-connected
verify Selene-connected message signature
match supplier record
check known supplier contact
detect fraud signals
apply payment hold under policy
notify AP and Payment
request missing evidence
send routine supplier status response
archive old bank ref after effective date
```

Selene escalates:

```text id="bank_escalates"
high fraud signal
supplier identity mismatch
beneficiary mismatch
bank country mismatch
new supplier first payment
active payment batch
urgent payment after bank change
manual override request
unverified non-Selene request
restricted supplier
large payment exposure
failed callback
```

Bank changes should be automated enough to reduce admin, but never casual enough to become an invitation.

---

## 28. PH1.D / GPT-5.5 Role

GPT-5.5 may help:

```text id="gpt_bank_allowed"
summarize bank-change request
draft supplier evidence request
explain fraud signals in plain English
draft internal review summary
draft supplier rejection message
summarize verification steps
translate supplier communications
prepare AP/Finance briefing
```

GPT-5.5 must not:

```text id="gpt_bank_forbidden"
approve bank change
create bank details
verify identity as final authority
override fraud hold
release payment
invent supplier authority
invent callback result
invent evidence
show full bank details to unauthorized users
```

GPT-5.5 can write the message.

It cannot open the money door.

Very strict. Very necessary.

---

## 29. Human-Like Selene Interaction

### Selene-connected bank change

> “Supplier ABC requested a bank change through their Selene. Their authorized finance user approved it. Buyer validation is pending, so I’ve placed payments on hold until the change is verified.”

### Non-Selene bank change

> “This supplier sent new bank details by email. I will not update payment details until I verify the request through the approved supplier contact.”

### Fraud signal

> “This request has fraud signals: new bank details on invoice, urgent payment language, and a bank country mismatch. I’m blocking payment and routing review.”

### Approved change

> “Supplier bank details are approved and will become active on 1 July. Payments before then will use the existing verified bank.”

### Payment blocked

> “I removed this supplier from today’s payment batch because their bank change is still under review.”

Human-like, direct, no panic, no waffle. Selene should sound like the AP manager who has seen every scam and no longer believes in email innocence.

---

## 30. State Machines

### Bank Change State

```text id="bank_change_state"
NotRequested
ChangeRequested
SupplierAuthorityVerifying
SupplierAuthorityVerified
BuyerValidationPending
BuyerApprovalPending
EvidenceRequested
ApprovedPendingEffectiveDate
Active
Rejected
Suspended
Cancelled
Archived
```

### Counterparty Trust State

```text id="counterparty_trust_state"
Unknown
KnownUnverified
ManualVerified
PortalVerified
SeleneConnected
SeleneConnectedStrongAuth
Suspended
Revoked
Archived
```

### Payment Safety State

```text id="payment_safety_state"
Safe
Watch
HoldRecommended
HoldApplied
Blocked
ExceptionApproved
Released
Archived
```

### Fraud Review State

```text id="fraud_review_state"
NoSignal
LowSignal
MediumSignal
HighSignal
ReviewRequired
FraudSuspected
Cleared
Blocked
Archived
```

### Supplier Portal Bank Change State

```text id="portal_bank_change_state"
Submitted
IdentityCheckPending
AuthorityCheckPending
EvidencePending
BuyerValidationPending
Approved
Rejected
Cancelled
Archived
```

---

## 31. Reason Codes

```text id="bank_reason_codes"
SUPPLIER_BANK_CHANGE_REQUESTED
SUPPLIER_BANK_CHANGE_FROM_SELENE
SUPPLIER_BANK_CHANGE_FROM_EMAIL
SUPPLIER_BANK_CHANGE_FROM_PORTAL
SUPPLIER_AUTHORITY_VERIFIED
SUPPLIER_AUTHORITY_FAILED
BUYER_SUPPLIER_MATCH_FAILED
COUNTERPARTY_TRUST_LEVEL_LOW
COUNTERPARTY_SIGNATURE_VALID
COUNTERPARTY_SIGNATURE_INVALID
REPLAY_PROTECTION_FAILED
BANK_CHANGE_EVIDENCE_REQUIRED
BANK_COUNTRY_MISMATCH
BENEFICIARY_NAME_MISMATCH
INVOICE_CONTAINS_NEW_BANK_DETAILS
URGENT_PAYMENT_WITH_BANK_CHANGE
PAYMENT_BATCH_HOLD_REQUIRED
SUPPLIER_PAYMENT_HOLD_APPLIED
BANK_CHANGE_APPROVED
BANK_CHANGE_REJECTED
BANK_CHANGE_EFFECTIVE_DATE_PENDING
BANK_CHANGE_ACTIVE
FRAUD_REVIEW_REQUIRED
CALLBACK_VERIFICATION_REQUIRED
CALLBACK_VERIFICATION_FAILED
```

---

## 32. Required Simulations

```text id="bank_simulations"
Selene-connected supplier requests bank change
supplier authority verification passes
supplier authority verification fails
buyer validates supplier Selene identity
bank change accepted under policy
bank change requires buyer approval
bank change rejected due to mismatch
non-Selene supplier sends bank change by email
callback verification succeeds
callback verification fails
invoice contains new bank details
urgent payment request after bank change
payment batch held due to bank change
bank change future effective date
supplier bank change with open AP invoices
supplier bank change with pending payment batch
fraud signal high blocks payment
supplier payment query with bank hold
counterparty trust key rotation
counterparty trust suspended
```

---

## 33. Integration Map

```text id="bank_integration_map"
PH1.SUPPLIER.BANK_TRUST / COUNTERPARTY_TRUST
↔ PH1.SUPPLIER
↔ PH1.CREDITORS / AP
↔ PH1.SUPPLIER_PAYMENT
↔ PH1.BANKING / PAYMENT_PROVIDER
↔ PH1.CASHFLOW
↔ PH1.PROCUREMENT
↔ PH1.PROC.RECEIVE
↔ PH1.CREDITORS.RECON
↔ PH1.LEGAL / CONTRACTS
↔ PH1.COMPLIANCE
↔ PH1.ACCESS / AUTHORITY
↔ PH1.AUDIT
↔ PH1.BCAST / DELIVERY
↔ PH1.REM
↔ PH1.WRITE
↔ PH1.D / GPT-5.5
```

---

## 34. Required Logical Packets

```text id="bank_packets"
SupplierBankChangeRequestPacket
SupplierBankChangeValidationPacket
SupplierBankChangeDecisionPacket
SupplierBankSafetyPacket
CounterpartyIdentityPacket
CounterpartyTrustPacket
SeleneToSeleneMessagePacket
SeleneToSeleneSignaturePacket
SupplierPortalBankChangePacket
ManualSupplierVerificationPacket
CallbackVerificationPacket
BankChangeFraudSignalPacket
SupplierPaymentHoldPacket
SupplierPaymentQueryPacket
SupplierRemittanceSafetyPacket
BankChangeAuditEvidencePacket
```

Logical only. Codex maps later. No packet structs yet, tiny schema gremlin.

---

## 35. What Codex Must Not Do

```text id="codex_no_banktrust"
Do not implement payment execution.
Do not create bank detail storage schema.
Do not expose raw bank details.
Do not let supplier messages mutate buyer records directly.
Do not let buyer-side staff casually edit supplier bank details.
Do not accept bank changes from invoice text.
Do not accept bank changes from unverified email.
Do not auto-release payment after bank change without policy.
Do not let GPT-5.5 approve bank changes.
Do not delete PH1.X or alter previous global documents.
Do not implement from this document alone.
```

---

## 36. Final Architecture Sentence

Selene Supplier Bank Change + Selene-to-Selene Counterparty Trust Protocol is the protected supplier-payment identity layer that requires supplier-originated bank changes, verifies supplier authority, validates Selene-connected counterparties, controls non-Selene manual verification, detects fraud signals, protects active payment batches, applies AP and payment holds, manages bank-change effective dates, restricts sensitive bank data, logs audit evidence, and allows trusted supplier messages such as credit notes, invoices, shipment updates, statements, and payment queries to flow between companies without letting any supplier, email, invoice, or GPT-5.5 output directly mutate buyer payment truth.

Simple version:

```text id="bank_simple"
Supplier changes their own bank details.
Their Selene verifies them.
Their Selene asks our Selene.
Our Selene validates.
Our policy decides.
Payments stay on hold if risky.
Emails and invoices cannot change bank details by themselves.
Supplier messages can help workflows but cannot directly change buyer truth.
Everything is audited.
```

That is Global Document 69. It is the lock on the supplier-payment door. Because supplier fraud does not need much — just one urgent email, one sleepy AP clerk, and one system dumb enough to believe a PDF wearing a logo.

[1]: https://www.ic3.gov/PSA/2024/PSA240911?utm_source=chatgpt.com "Business Email Compromise: The $55 Billion Scam"
[2]: https://www.acfe.com/fraud-magazine/all-issues/issue/article?s=2017-julyaug-fraud-basics&utm_source=chatgpt.com "Poor bank account management yields fraud"
