# Global Document 75 — Selene Supplier Statement Reconciliation + Creditor Reporting Engine

```text id="doc75_status"
DOCUMENT TYPE:
GLOBAL MASTER ARCHITECTURE DESIGN

GLOBAL DOCUMENT NUMBER:
75

ENGINE:
PH1.CREDITORS.RECON / PH1.SUPPLIER_STATEMENT_RECON / PH1.CREDITOR_REPORTING

FULL NAME:
Selene Supplier Statement Reconciliation, Creditor Balance, Vendor Statement Matching, Unapplied Credit, Unapplied Payment, AP Close, Creditor Reporting, and Supplier Balance Control Engine

STATUS:
FUTURE_CANONICAL_ARCHITECTURE
NOT_RUNTIME_IMPLEMENTATION
PENDING_REPO_TRUTH_ACTIVATION
PENDING_SIMULATION_MAPPING
CODEX_READY_MASTER_DESIGN
```

---

## 1. Purpose

Selene Supplier Statement Reconciliation + Creditor Reporting Engine owns the reconciliation of supplier claims against Selene’s creditor truth.

It answers:

```text id="supplier_recon_questions"
Does the supplier statement match our AP records?
Are they claiming invoices we do not recognize?
Are they missing our credit notes?
Are they showing invoices already paid?
Are they still claiming damaged or short goods?
Are there unapplied credits?
Are there unapplied payments?
Are refunds missing?
Are replacements unresolved?
Is the supplier balance clean?
Is the aged creditors report accurate?
Is month-end AP close ready?
Which supplier accounts need action?
What should Selene send back to the supplier?
```

Supplier statements are **not truth**.

Supplier statements are **claims**.

Selene checks those claims against:

```text id="supplier_recon_truth_sources"
Supplier Invoice records
Purchase Orders
Receiving proof
Inspection acceptance
Credit notes
Replacement obligations
Refund obligations
Supplier payments
BankRec confirmations
AP holds
Supplier obligations
Supplier disputes
Accounting evidence
```

A supplier statement says:

```text id="supplier_statement_claim"
You owe us $12,000.
```

Selene says:

```text id="selene_supplier_recon_answer"
We agree $9,700 is clean payable.
$1,300 was already paid.
$600 is held pending damaged-goods credit note.
$400 is an unknown invoice claim with no PO, receiving, or AP record.
```

That is creditor control.

Not “supplier sent a PDF, so everyone gather around the panic table.”

---

## 2. Why Supplier Statement Reconciliation Comes After Supplier Payment

The chain is:

```text id="supplier_statement_chain"
Supplier
→ Procurement / Purchase Order
→ Receiving / Inspection
→ AP / Creditors
→ Supplier Payment
→ BankRec / Treasury
→ Supplier Statement Reconciliation
→ Creditor Reporting
→ Accounting / Period Close
```

AP owns invoice-level payable truth.

Supplier Payment owns payment execution handoff and remittance.

BankRec proves actual bank movement.

Supplier Statement Reconciliation compares supplier statements against all of that proof.

Boundary:

```text id="statement_boundary"
AP decides invoice truth.
Supplier Payment proves payment instruction/submission/remittance.
BankRec proves bank clearing.
Supplier Statement Reconciliation proves whether supplier balance claims match Selene truth.
```

Do not merge Statement Reconciliation into AP.

Do not pay from supplier statements.

Do not let supplier statements overwrite AP.

Supplier statements are the supplier’s version of reality. Sometimes useful. Sometimes creative. Always verify.

---

## 3. Modern Reconciliation Readiness

Supplier reconciliation must connect AP matching, remittance, payment proof, and structured transaction data. CIPS describes invoice verification against the purchase order and goods receipt as a critical control mechanism to prevent errors and mitigate fraud risk, which means supplier statements should be reconciled against PO, receiving, invoice, and payment truth instead of being treated as standalone evidence. ([CIPS Download][1])

Structured payment data also matters. Swift says ISO 20022 helps corporates reduce payment friction and streamline reconciliation through richer, structured remittance information, so Selene must be ready to use invoice references, credit notes, remittance details, and payment status messages in supplier reconciliation. ([Swift][2])

Practical supplier statement reconciliation compares supplier statement lines against AP ledger data, including invoices, payments, and credit notes. That is the business pattern Selene must automate fully instead of making humans tick rows manually like monks sorting receipts by candlelight. ([Aurum Solutions][3])

Translation:

```text id="modern_recon_translation"
Supplier reconciliation is not manual statement ticking.
It is claim matching across AP, PO, receiving, credit, payment, bank, and accounting evidence.
```

---

## 4. Core Selene Law

```text id="supplier_recon_core_law"
Supplier statements are claims, not truth.

Selene must automatically reconcile supplier statement claims against AP, Procurement, Receiving, Supplier Obligations, Supplier Payment, BankRec, Credit Notes, Refunds, Replacements, and Accounting evidence.

Routine matches close automatically.
Clear mismatches are explained and routed automatically.
Humans review only material unresolved exceptions, supplier disputes, write-offs, settlements, and policy overrides.
```

Selene must reduce human work by:

```text id="supplier_recon_reduce_work"
capturing supplier statements
extracting statement lines
matching statement invoices
matching statement payments
matching credit notes
matching refunds
matching supplier obligations
identifying unknown invoices
identifying missing payments
identifying missing credits
identifying duplicate supplier claims
identifying unapplied credits
identifying unapplied payments
drafting supplier responses
sending routine reconciliation replies
preparing aged creditors
preparing AP close packs
preparing supplier balance confirmations
updating Supplier score
```

Humans should not manually compare a supplier PDF to AP rows like they are decoding ancient ruins.

Selene should do the archaeology. Humans can review the mummies that bite.

---

## 5. Engine Boundary

### 5.1 PH1.CREDITORS.RECON owns

```text id="creditors_recon_owns"
supplier statement intake
supplier statement extraction
supplier statement line classification
supplier statement-to-AP matching
supplier invoice claim matching
supplier payment claim matching
supplier credit note claim matching
supplier refund claim matching
supplier obligation matching
unapplied credit detection
unapplied payment detection
unknown invoice detection
duplicate supplier claim detection
supplier balance comparison
supplier balance dispute generation
aged creditors reporting
creditor reporting snapshots
AP close support
supplier balance confirmation
supplier reconciliation response drafting
routine supplier response sending under policy
supplier reconciliation exception worklist
supplier reconciliation audit evidence
```

### 5.2 PH1.CREDITORS.RECON does not own

```text id="creditors_recon_not_own"
supplier invoice validation
supplier payment execution
bank payment settlement proof
purchase order creation
goods receiving truth
supplier bank change approval
ledger posting
tax treatment
supplier qualification truth
final write-off approval
legal settlement
```

### 5.3 Correct owner split

```text id="creditors_recon_owner_split"
PH1.CREDITORS / AP = invoice-level payable truth and AP holds.
PH1.SUPPLIER_PAYMENT = payment scheduling, remittance, provider submission status.
PH1.BANKREC / TREASURY = bank clearing proof.
PH1.SUPPLIER = supplier identity, score, obligations, disputes.
PH1.PROCUREMENT = PO proof.
PH1.PROC.RECEIVE = receiving and inspection proof.
PH1.CREDITORS.RECON = supplier statement vs Selene truth.
PH1.ACCOUNTING = ledger, AP control account, financial reporting.
PH1.PERIOD_CLOSE = close readiness and reporting pack.
PH1.AUDIT = evidence.
```

Supplier Reconciliation is the creditor balance judge.

It does not create invoice truth.

It does not move money.

It does not post books.

It tells everyone whether the supplier’s claimed balance agrees with reality.

A noble task. Slightly thankless. Very necessary.

---

## 6. Supplier Statement Sources

Selene receives supplier statements from:

```text id="statement_sources"
supplier email
supplier portal
Supplier Selene
PDF upload
photo/scanned statement
spreadsheet
EDI / e-invoice platform
B2B platform
supplier accounting system export
monthly statement pack
manual upload
```

Statement types:

```text id="statement_types"
monthly supplier statement
weekly supplier statement
on-demand supplier statement
payment chase statement
credit control statement
year-end balance confirmation
supplier dispute statement
supplier remittance mismatch notice
```

Selene should say:

> “Upload the supplier statement. I’ll compare it to our AP, payments, credits, and receiving disputes.”

Not:

> “Please manually tick 94 lines.”

We are not trying to preserve clerical suffering for heritage reasons.

---

## 7. Supplier Statement Master Record

Every supplier statement creates a controlled record.

```text id="supplier_statement_record"
supplier_statement_id
legal_entity_id
supplier_id
supplier_name_on_statement
statement_date
statement_period_start
statement_period_end
currency
opening_balance_claimed
closing_balance_claimed
statement_total_invoices
statement_total_credits
statement_total_payments
statement_total_balance
source_channel
source_document_ref
extraction_status
supplier_match_status
line_count
matched_line_count
unmatched_line_count
variance_amount
materiality_status
reconciliation_status
exception_count
supplier_response_status
audit_ref
```

Statement statuses:

```text id="supplier_statement_statuses"
Received
ExtractionPending
Extracted
SupplierMatching
SupplierMatched
LineMatching
FullyMatched
PartiallyMatched
ExceptionsDetected
SupplierResponsePrepared
SupplierResponseSent
AwaitingSupplierResponse
Resolved
CloseReady
Closed
Archived
```

Selene should always know:

```text id="statement_summary_truth"
what supplier claims
what Selene agrees
what Selene rejects
what evidence supports the difference
```

Supplier statement without reconciliation is just an invoice family reunion.

---

## 8. Supplier Statement Line Model

Each statement line becomes a line record.

```text id="statement_line_record"
statement_line_id
supplier_statement_id
line_type
supplier_reference
invoice_number
credit_note_number
payment_reference
transaction_date
due_date
amount
currency
description
claimed_open_amount
matched_internal_ref
matched_engine
match_confidence
match_status
variance_amount
reason_code
audit_ref
```

Line types:

```text id="statement_line_types"
InvoiceClaim
CreditNoteClaim
PaymentClaim
OpeningBalance
Adjustment
Refund
Discount
InterestCharge
FeeCharge
BalanceForward
Unknown
```

Line statuses:

```text id="statement_line_statuses"
Unmatched
Matched
PartiallyMatched
UnknownClaim
DuplicateClaim
AlreadyPaid
CreditMissing
PaymentMissingFromSupplier
PaymentUnapplied
CreditUnapplied
Disputed
OnHold
Rejected
Closed
```

This lets Selene reconcile line by line.

Not just total against total.

Total-to-total reconciliation is how bad details hide under a nice number.

---

## 9. Statement Extraction

Selene extracts:

```text id="statement_extraction_fields"
supplier name
supplier account number
statement date
period
opening balance
invoice lines
credit note lines
payment lines
adjustment lines
closing balance
currency
contact details
bank details shown
due/overdue indicators
references
```

If supplier statement contains new bank details:

```text id="statement_bank_detail_warning"
Do not update bank details.
Send to Supplier Bank Trust.
Apply payment hold if required.
```

Selene says:

> “The statement includes bank details that differ from our verified supplier record. I will not use them and will route bank safety review.”

Statements are not bank-change authority. Repeat until fraud gets bored.

---

## 10. Supplier Matching

Statement must match known supplier.

Supplier matching checks:

```text id="statement_supplier_matching"
supplier legal name
trading name
supplier account number
supplier tax ID
supplier email/domain
supplier portal identity
Supplier Selene identity
existing supplier_id
bank details warning only, not match authority
```

Outcomes:

```text id="supplier_match_outcomes"
MatchedSupplier
PossibleSupplierMatch
UnknownSupplierStatement
DuplicateSupplierRisk
FraudReviewRequired
```

Selene says:

> “This statement may belong to Supplier ABC, but the supplier account number differs. I’ll hold matching until verified.”

A statement from an unknown supplier is not a creditor balance.

It is a question with a total at the bottom.

---

## 11. Invoice Claim Matching

Supplier statement invoice lines match AP invoice records.

Match keys:

```text id="invoice_claim_match_keys"
supplier_id
invoice_number
invoice_date
invoice_amount
currency
PO_ref
AP_invoice_id
line descriptions
statement open amount
tax amount
```

Outcomes:

```text id="invoice_match_outcomes"
InvoiceMatched
InvoicePartiallyMatched
InvoiceAlreadyPaid
InvoiceOnHold
InvoiceDisputed
InvoiceCancelled
UnknownInvoice
DuplicateInvoiceClaim
AmountMismatch
CurrencyMismatch
```

Selene says:

> “The supplier statement includes INV-884, which matches our AP invoice and is scheduled for payment.”

Or:

> “The statement includes INV-992, but Selene has no matching invoice, PO, or receiving proof. I’ll request a copy and keep it out of clean payable balance.”

Unknown invoice claim does not become AP just because it appears on a statement.

Suppliers do not get to create invoices through statement osmosis. Tragic for them.

---

## 12. Payment Claim Matching

Supplier statements often show payments received or missing.

Selene matches statement payment lines to:

```text id="payment_claim_sources"
Supplier Payment records
payment batch records
remittance advice
provider references
BankRec clearing proof
bank references
supplier Selene payment receipt
```

Outcomes:

```text id="payment_match_outcomes"
PaymentMatched
SupplierMissingPayment
PaymentUnappliedBySupplier
PaymentAmountMismatch
PaymentDateMismatch
PaymentFailedInternally
PaymentNotYetCleared
PaymentUnknown
```

Selene says:

> “Supplier statement does not show our payment from 14 May. BankRec confirms it cleared, and remittance was sent. I’ll prepare payment proof for the supplier.”

Supplier not applying payment correctly is not our unpaid invoice.

It is their admin problem, now documented.

---

## 13. Credit Note Claim Matching

Supplier statement credit lines must match AP credit note records and supplier obligations.

Match keys:

```text id="credit_match_keys"
credit_note_number
supplier_id
original_invoice_ref
PO_ref
receiving_ref
supplier_obligation_ref
amount
currency
tax amount
reason
```

Outcomes:

```text id="credit_match_outcomes"
CreditMatched
CreditExpectedNotReceived
CreditMissingFromSupplierStatement
CreditAppliedWrongInvoice
CreditAmountMismatch
CreditNoteRejected
CreditUnapplied
```

Selene says:

> “The supplier statement still includes $200 for damaged goods. We requested credit note CN-221, but supplier has not applied it. I’ll resend the evidence and keep AP hold active.”

Credit notes should not vanish into supplier accounting fog.

Selene brings a torch and, unfortunately for them, receipts.

---

## 14. Refund Matching

Refunds may appear on supplier statements or bank records.

Selene matches refunds to:

```text id="refund_match_sources"
refund obligation
supplier refund confirmation
BankRec receipt
AP refund receivable
supplier statement refund line
settlement agreement
```

Outcomes:

```text id="refund_match_outcomes"
RefundMatched
RefundExpectedNotReceived
RefundReceivedNotApplied
RefundAmountMismatch
RefundClaimedButNoBankProof
```

Selene says:

> “Supplier claims refund was issued, but BankRec has no matching receipt. I’ll keep the refund obligation open.”

Once again: words are not cash.

Even supplier statement words wearing a balance column.

---

## 15. Supplier Obligation Matching

Supplier statement must be checked against open obligations.

Obligation types:

```text id="obligation_types_for_recon"
credit note owed
replacement owed
refund owed
short delivery unresolved
damaged goods unresolved
wrong goods unresolved
service rework unresolved
warranty claim
overcharge correction
```

Selene checks:

```text id="obligation_matching"
does supplier statement include disputed amount?
does supplier statement reflect credit?
does supplier statement ignore replacement?
does supplier statement claim payment for rejected goods?
does supplier statement still show invoice under AP hold?
```

Selene says:

> “Supplier statement includes an invoice amount that is currently on AP hold due to unresolved receiving dispute. I’ll classify it as disputed, not clean payable.”

Supplier statements do not erase supplier obligations. Nice try, PDF.

---

## 16. Unknown Invoice Claims

Unknown invoice claims are dangerous.

Unknown invoice if:

```text id="unknown_invoice_conditions"
not in AP
no matching PO
no matching receiving
no matching contract
no recurring bill record
no approved exception
not in invoice intake queue
```

Selene actions:

```text id="unknown_invoice_actions"
classify UnknownInvoice
exclude from clean payable
request invoice copy/evidence from supplier
search inbox/portal
ask Procurement if authorized only if material
apply risk score
open exception
```

Selene says:

> “Supplier claims invoice INV-992, but Selene has no AP, PO, receiving, or contract record. I’ve excluded it from payable balance and requested evidence.”

No invoice can be born from a statement line alone. That would be financial spontaneous generation. The church tried worse ideas.

---

## 17. Duplicate Statement Claims

Supplier may list the same invoice twice or claim an already settled invoice.

Duplicate signals:

```text id="statement_duplicate_signals"
same invoice number appears twice
same amount/date/reference duplicated
invoice previously paid
invoice cancelled/reversed
credit note ignored
same invoice on prior statement already disputed
statement includes both original and corrected invoice
```

Actions:

```text id="statement_duplicate_actions"
flag duplicate claim
exclude duplicate from clean balance
send supplier correction request
update Supplier statement accuracy score
```

Selene says:

> “Invoice INV-884 appears twice on the supplier statement. I’ve treated the second line as duplicate and excluded it from payable balance.”

Supplier statement duplicates are not “extra payable.” They are supplier accounting hiccups with ambition.

---

## 18. Unapplied Credits

Selene detects credits that supplier has issued but not applied correctly, or credits that exist internally but supplier statement ignores.

Unapplied credit examples:

```text id="unapplied_credit_examples"
credit note exists but supplier balance still includes full invoice
credit note applied to wrong invoice
credit note amount differs from obligation
credit note exists in AP but missing from supplier statement
supplier statement shows credit but AP cannot match it
```

Actions:

```text id="unapplied_credit_actions"
link credit to obligation
prepare supplier correction request
keep AP balance correct
do not overpay
update statement exception
```

Selene says:

> “Supplier issued a $200 credit note but did not apply it to the damaged-goods invoice. I’ll request correction and keep our payable balance reduced.”

Credits are money. Lost credits are tiny leaks. Selene plugs leaks.

---

## 19. Unapplied Payments

Selene detects payments that supplier has not applied properly.

Unapplied payment examples:

```text id="unapplied_payment_examples"
supplier statement still shows paid invoice open
payment applied to wrong invoice
supplier records payment short
supplier missing remittance detail
bank/provider confirmed payment but supplier not updated
```

Actions:

```text id="unapplied_payment_actions"
attach remittance
attach bank proof
send supplier correction request
keep AP marked paid
open supplier statement exception
update supplier admin score
```

Selene says:

> “Supplier has not applied our $4,800 payment. I’ll send remittance and bank proof.”

No paying again because supplier lost the first payment in their own system. This is business, not hide-and-seek.

---

## 20. Supplier Balance Calculation

Selene calculates supplier balance from its own truth.

Formula:

```text id="supplier_balance_formula"
opening AP balance
+ approved supplier invoices
- credit notes applied
- payments confirmed
- refunds received/applied
+/- approved adjustments
= Selene calculated supplier balance
```

Supplier statement balance:

```text id="supplier_statement_balance"
supplier claimed opening balance
+ supplier claimed invoices
- supplier claimed credits
- supplier claimed payments
+/- supplier adjustments
= supplier claimed closing balance
```

Variance:

```text id="balance_variance"
supplier claimed balance - Selene calculated balance
```

Variance categories:

```text id="variance_categories"
clean match
timing difference
supplier missing payment
supplier missing credit
unknown invoice
duplicate claim
disputed AP hold
refund mismatch
currency/FX difference
material unexplained variance
```

Selene says:

> “Supplier claims $12,000. Selene calculates $9,700 clean payable. Difference is explained by $1,300 paid, $600 credit pending, and $400 unknown invoice.”

That is reconciliation. Not just saying “variance $2,300” and wandering off like a coward.

---

## 21. Materiality and Auto-Close Rules

Selene should auto-close clean or immaterial reconciliations under policy.

Materiality inputs:

```text id="materiality_inputs"
absolute variance amount
percentage of supplier balance
supplier risk
supplier criticality
dispute history
period close status
cashflow impact
audit policy
```

Auto-close allowed when:

```text id="auto_close_allowed"
all lines matched
variance zero
or immaterial timing-only variance under policy
no supplier dispute
no high-risk supplier
no missing credit/payment
evidence complete
```

Human review required when:

```text id="human_review_required"
material variance
unknown invoice above threshold
supplier disputes evidence
legal threat
write-off or settlement
fraud signal
manual override
period close blocker
```

Selene says:

> “Supplier statement matches our records. I’ve closed the reconciliation.”

Or:

> “Variance is $8.40 bank fee timing and below threshold. I’ll close under policy.”

Do not summon a human over $8.40 unless the human has personally offended the reconciliation gods.

---

## 22. Supplier Response Automation

Selene prepares and may send routine supplier responses.

Response types:

```text id="supplier_response_types"
statement matched confirmation
payment proof response
missing credit note request
unknown invoice request
duplicate invoice correction
balance dispute response
unapplied payment correction
unapplied credit correction
supplier statement exception summary
```

Examples:

```text id="supplier_response_examples"
“Your statement matches our records.”

“Invoice INV-884 was paid on 14 May. Remittance and bank reference attached.”

“Your statement still includes $200 for damaged goods from PO-771. Credit note was requested and evidence is attached.”

“We cannot match invoice INV-992 to any PO, receiving event, or AP record. Please send supporting documents.”
```

GPT-5.5 may draft messages.

Selene must source facts from deterministic reconciliation records.

No poetic supplier disputes. We need facts, not a strongly worded hallucination.

---

## 23. Supplier Statement Chasing

Selene should request missing statements automatically.

Statement request cadence:

```text id="statement_request_cadence"
critical/high-volume suppliers = monthly or more frequent
normal suppliers = monthly
low-volume suppliers = quarterly or on-demand
high-risk suppliers = every statement cycle and after disputes
period close = all material suppliers
```

Chase triggers:

```text id="statement_chase_triggers"
month-end close approaching
supplier statement missing
supplier dispute open
supplier high-volume activity
AP close requires balance proof
auditor/accountant request
```

Selene says:

> “Three material supplier statements are missing for month-end. I’ll request them now.”

AP close should not be blocked because someone forgot to ask a supplier for a statement. Selene can nag. Selene was built for this.

---

## 24. Reconciliation Frequency

Selene runs reconciliation by both event and schedule.

Event-driven triggers:

```text id="recon_event_triggers"
supplier statement received
supplier sends payment query
credit note received
payment confirmed
refund received
supplier disputes balance
month-end close starts
supplier risk increases
large invoice received
supplier bank change
```

Scheduled cadence:

```text id="recon_scheduled_cadence"
critical suppliers = weekly or monthly
high-volume suppliers = weekly/monthly
normal suppliers = monthly
low-volume suppliers = quarterly/on statement
high-risk suppliers = every statement
period close = all material suppliers
```

Selene says:

> “Supplier ABC is high-volume and has statement mismatches. I will reconcile weekly until clean for two cycles.”

Automation should adapt to risk. Not everything needs the same rhythm. This is reconciliation, not a metronome cult.

---

## 25. Aged Creditors Reporting

Selene produces aged creditors.

Buckets:

```text id="aged_creditors_buckets"
Current
1–30 days
31–60 days
61–90 days
90+ days
OnHold
Disputed
CreditPending
PaymentScheduled
PaidNotAppliedBySupplier
UnknownClaim
```

But Selene must avoid dumb aging.

Selene must separate:

```text id="aged_creditors_separation"
clean payable
scheduled payment
valid AP hold
supplier dispute
missing credit
supplier missing payment
unknown invoice
bank-risk hold
cashflow-scheduled delay
```

Example:

```text id="aged_creditors_example"
Supplier ABC:
Statement claim: $12,000
Clean payable: $9,700
Scheduled: $8,000
On AP hold: $600
Paid but supplier not applied: $1,300
Unknown invoice: $400
```

That is aged creditors with intelligence.

Not “overdue $12,000” because the supplier PDF said so. That is how reports lie politely.

---

## 26. Creditor Reporting

Selene produces creditor reports.

Reports:

```text id="creditor_reports"
Supplier Balance Summary
Aged Creditors
Supplier Statement Reconciliation Report
Open Supplier Statement Exceptions
Unknown Supplier Invoice Claims
Supplier Missing Payment Report
Supplier Missing Credit Note Report
Unapplied Credit Report
Unapplied Payment Report
Supplier Dispute Report
Credit Note Aging Report
Refunds Pending Report
Supplier Obligation Report
AP Close Creditor Pack
Supplier Confirmation Pack
```

Selene explains:

> “Total creditors are $420,000, but clean payable is $310,000. $62,000 is scheduled, $31,000 is disputed, and $17,000 is pending supplier credits.”

That is the kind of report management can understand without sacrificing a junior accountant to Excel.

---

## 27. Credit Note Aging

Selene ages expected supplier credits.

Buckets:

```text id="credit_note_aging_buckets"
Requested
0–7 days pending
8–14 days pending
15–30 days pending
31+ days pending
Escalated
Received
Matched
Closed
```

Selene tracks:

```text id="credit_note_aging_fields"
supplier
PO
invoice
receiving event
reason
amount
request date
due date
days outstanding
supplier response
AP hold
escalation status
```

Selene says:

> “Supplier ABC has three credit notes overdue beyond 30 days. I recommend restricting new orders until resolved.”

A supplier that does not issue credits is using your cash as furniture. Selene notices.

---

## 28. Refund Aging

Refunds pending also need aging.

Buckets:

```text id="refund_aging_buckets"
Requested
Expected
Overdue0To7
Overdue8To14
Overdue15To30
OverdueOver30
Escalated
Received
Matched
Closed
```

Selene says:

> “Supplier refund for overpaid invoice has been pending for 18 days. I recommend escalation.”

Refunds should not become bedtime stories.

---

## 29. Supplier Balance Confirmation

Selene supports formal supplier balance confirmation.

Used for:

```text id="balance_confirmation_use"
month-end
year-end
audit
external accountant review
supplier dispute
creditor control
material supplier review
```

Flow:

```text id="balance_confirmation_flow"
select supplier
prepare Selene balance
send confirmation request
supplier responds
match response
exceptions listed
close confirmation or create dispute
```

Selene says:

> “Supplier confirmed a different balance. I reconciled the difference to one missing payment and one unknown invoice.”

Balance confirmation is not just “supplier agreed.” It is evidence with reconciliation.

---

## 30. AP Close Support

Period Close needs creditor readiness.

Selene provides:

```text id="ap_close_support"
material supplier statements received
material supplier statements reconciled
open supplier statement exceptions
credit notes pending
refunds pending
unapplied payments
unapplied credits
unknown invoice claims
supplier statement disputes
aged creditors
AP control support
```

Close readiness states:

```text id="ap_close_recon_state"
Ready
MinorExceptions
MaterialExceptions
Blocked
```

Selene says:

> “AP close is not blocked. Two supplier statement exceptions are immaterial and disclosed. One material supplier statement is still missing.”

This feeds:

```text id="period_close_link"
Global Document 14 — Period Close + Financial Reporting Engine
```

Period Close owns final close.

Supplier Reconciliation provides creditor close evidence.

Everybody, yet again, remains in their chair. The chairs are labelled for a reason.

---

## 31. Supplier Score Impact

Supplier statement behavior affects Supplier Intelligence.

Events sent to Supplier Engine:

```text id="supplier_score_events"
statement clean
statement late/missing
unknown invoice claim
duplicate invoice claim
missing credit note
missing payment
unapplied payment
unapplied credit
supplier disputes valid evidence
supplier resolves promptly
supplier response slow
```

Supplier score impacts:

```text id="score_impacts"
statement accuracy score
admin reliability score
credit note reliability score
dispute behavior score
response score
overall trust score
watchlist/restriction recommendation
```

Selene says:

> “Supplier admin reliability score reduced due to repeated unknown invoice claims.”

Bad admin is supplier risk. It wastes AP time and can create payment errors. Selene should score it.

---

## 32. Supplier Statement and Bank Details

Statements may contain bank details.

Rules:

```text id="statement_bank_rules"
supplier statement bank details are informational only
do not update supplier bank from statement
if bank details differ, send to Supplier Bank Trust
apply payment hold if policy requires
do not include unverified bank details in remittance
```

Selene says:

> “Supplier statement shows different bank details. I will not use them. I’ve routed bank-safety review.”

Statements are not bank-change documents.

Fraud loves statements too. Fraud is very industrious.

---

## 33. Selene-to-Selene Statement Protocol

If supplier is Selene-connected, statements can be exchanged as structured packets.

Supplier Selene sends:

```text id="supplier_selene_statement_packets"
SupplierStatementPacket
SupplierInvoiceListPacket
SupplierCreditNotePacket
SupplierPaymentAcknowledgementPacket
SupplierBalanceQueryPacket
```

Buyer Selene responds:

```text id="buyer_selene_statement_packets"
StatementReconciliationResultPacket
MatchedStatementLinesPacket
StatementExceptionLinesPacket
PaymentProofPacket
CreditNoteDisputePacket
SupplierBalanceConfirmationPacket
```

Example:

```text id="selene_to_selene_recon_example"
Supplier Selene:
Closing balance claimed = $12,000

Buyer Selene:
Clean payable = $9,700
Already paid = $1,300
Credit pending = $600
Unknown invoice = $400
Evidence attached
```

Two Selenes should reconcile faster than two humans with seven PDFs and one passive-aggressive “as per my last email.”

---

## 34. Human Approval and Exception Rules

Selene auto-handles:

```text id="recon_auto_handles"
fully matched supplier statements
immaterial timing differences under policy
supplier missing payment proof response
supplier missing credit note response
unknown invoice evidence request
duplicate invoice correction request
routine statement chase
routine aged creditor update
routine AP close pack update
```

Selene escalates:

```text id="recon_escalates"
material unexplained variance
supplier disputes bank proof
unknown invoice above threshold
legal demand / payment threat
write-off or settlement
supplier relationship risk
suspected fraud
manual override
period close blocker
tax/accounting policy conflict
```

Rule:

```text id="recon_exception_rule"
Routine match = Selene closes.
Routine mismatch = Selene responds.
Material unresolved issue = human review.
Settlement/write-off/legal = authority.
Everything = audited.
```

No human should manually review a clean supplier statement.

No robot should write off a material balance because it looked tedious.

Balance. Again. The theme of the century.

---

## 35. PH1.D / GPT-5.5 Role

GPT-5.5 may help:

```text id="gpt_recon_allowed"
summarize supplier statement
explain reconciliation variance
draft supplier payment proof response
draft missing credit note request
draft unknown invoice request
draft supplier balance dispute response
summarize aged creditors
draft AP close commentary
translate supplier messages
prepare creditor report narrative
```

GPT-5.5 must not:

```text id="gpt_recon_forbidden"
accept supplier statement as truth
write off variance
invent payment proof
invent credit note
alter AP balance
close material dispute
settle supplier claim
post ledger
override audit
```

GPT may write:

> “The balance difference relates to one missing credit note and one payment already made.”

GPT may not decide:

> “Let’s just pay it.”

No, little language model. The supplier already has enough ideas.

---

## 36. Human-Like Selene Interaction

### Clean statement

> “Supplier ABC’s statement matches our records. I’ve closed the reconciliation.”

### Missing payment

> “Supplier ABC does not show our payment from 14 May. BankRec confirms it cleared, so I’ll send remittance and bank proof.”

### Missing credit note

> “Supplier ABC still shows the damaged-goods amount. I’ll resend the receiving evidence and request the credit note again.”

### Unknown invoice

> “This statement includes invoice INV-992, which I cannot match to any PO, receiving event, or AP record. I’ll ask the supplier for supporting documents.”

### Aged creditors

> “Total supplier claims are $420,000. Clean payable is $310,000. The difference is made up of scheduled payments, disputes, and pending credits.”

### AP close

> “AP close is nearly ready. One material supplier statement is still missing and two credit notes are overdue.”

Human-like, precise, not “variance exception state unresolved.” Nobody wants to be emotionally harmed by a report.

---

## 37. State Machines

### Supplier Statement State

```text id="supplier_statement_state"
Received
ExtractionPending
Extracted
SupplierMatching
SupplierMatched
LineMatching
FullyMatched
PartiallyMatched
ExceptionsDetected
SupplierResponsePrepared
SupplierResponseSent
AwaitingSupplierResponse
Resolved
CloseReady
Closed
Archived
```

### Statement Line State

```text id="statement_line_state"
Unmatched
Matched
PartiallyMatched
UnknownClaim
DuplicateClaim
AlreadyPaid
CreditMissing
PaymentMissingFromSupplier
PaymentUnapplied
CreditUnapplied
Disputed
OnHold
Rejected
Closed
```

### Supplier Balance Dispute State

```text id="supplier_balance_dispute_state"
Detected
EvidenceAttached
SupplierContacted
AwaitingSupplier
SupplierResponded
InternalReview
Accepted
Rejected
Escalated
Resolved
Closed
Archived
```

### Credit Note Aging State

```text id="credit_note_aging_state"
Expected
Requested
Pending0To7
Pending8To14
Pending15To30
PendingOver30
Escalated
Received
Matched
Closed
```

### Refund Aging State

```text id="refund_aging_state"
Requested
Expected
Overdue0To7
Overdue8To14
Overdue15To30
OverdueOver30
Escalated
Received
Matched
Closed
```

### AP Close Creditor State

```text id="ap_close_creditor_state"
NotStarted
StatementsRequested
Reconciling
Ready
MinorExceptions
MaterialExceptions
Blocked
Closed
Archived
```

---

## 38. Reason Codes

```text id="supplier_recon_reason_codes"
SUPPLIER_STATEMENT_RECEIVED
SUPPLIER_STATEMENT_EXTRACTED
SUPPLIER_MATCHED
SUPPLIER_STATEMENT_FULLY_MATCHED
SUPPLIER_STATEMENT_VARIANCE
UNKNOWN_SUPPLIER_INVOICE_CLAIM
DUPLICATE_SUPPLIER_INVOICE_CLAIM
SUPPLIER_MISSING_PAYMENT
SUPPLIER_MISSING_CREDIT_NOTE
UNAPPLIED_SUPPLIER_CREDIT
UNAPPLIED_SUPPLIER_PAYMENT
SUPPLIER_CREDIT_AMOUNT_MISMATCH
SUPPLIER_PAYMENT_AMOUNT_MISMATCH
SUPPLIER_REFUND_EXPECTED
SUPPLIER_REFUND_MISSING
SUPPLIER_BALANCE_CONFIRMED
SUPPLIER_BALANCE_DISPUTED
CREDIT_NOTE_OVERDUE
REFUND_OVERDUE
STATEMENT_MISSING
STATEMENT_RESPONSE_SENT
PAYMENT_PROOF_SENT_TO_SUPPLIER
UNKNOWN_INVOICE_EVIDENCE_REQUESTED
AP_CLOSE_CREDITOR_READY
AP_CLOSE_CREDITOR_BLOCKED
MATERIAL_VARIANCE_REVIEW_REQUIRED
SUPPLIER_ADMIN_SCORE_UPDATE_REQUIRED
BANK_DETAILS_ON_STATEMENT_DIFFER
SUPPLIER_BANK_REVIEW_REQUIRED
```

---

## 39. Required Simulations

```text id="supplier_recon_simulations"
supplier statement fully matched
supplier statement with unknown invoice
supplier statement duplicate invoice
supplier statement missing our payment
supplier statement missing credit note
supplier statement includes disputed AP hold
unapplied supplier credit detected
unapplied supplier payment detected
credit note amount mismatch
refund expected but not received
payment proof sent to supplier
unknown invoice evidence requested
supplier balance confirmation clean
supplier balance disagreement
statement missing at month-end
credit note aging overdue
refund aging overdue
AP close creditor ready
AP close blocked by material supplier variance
Supplier Selene sends statement packet
Buyer Selene sends reconciliation result
supplier admin score updated
statement bank details differ from verified supplier bank
```

---

## 40. Integration Map

```text id="supplier_recon_integration_map"
PH1.CREDITORS.RECON / SUPPLIER_STATEMENT_RECON
↔ PH1.CREDITORS / AP
↔ PH1.SUPPLIER
↔ PH1.SUPPLIER.BANK_TRUST
↔ PH1.SUPPLIER_PAYMENT
↔ PH1.BANKREC / TREASURY
↔ PH1.PROCUREMENT / PH1.PROC.ORDER
↔ PH1.PROC.RECEIVE
↔ PH1.INVENTORY
↔ PH1.ACCOUNTING
↔ PH1.PERIOD_CLOSE / FIN_REPORTING
↔ PH1.CASHFLOW
↔ PH1.BUDGET
↔ PH1.TAX
↔ PH1.TAX.OPTIMIZE
↔ PH1.LEGAL / CONTRACTS
↔ PH1.ACCESS / AUTHORITY
↔ PH1.AUDIT
↔ PH1.REM
↔ PH1.BCAST / DELIVERY
↔ PH1.WRITE
↔ PH1.D / GPT-5.5
```

---

## 41. Required Logical Packets

```text id="supplier_recon_packets"
SupplierStatementIntakePacket
SupplierStatementExtractionPacket
SupplierStatementPacket
SupplierStatementLinePacket
SupplierStatementLineMatchPacket
SupplierInvoiceClaimMatchPacket
SupplierPaymentClaimMatchPacket
SupplierCreditNoteClaimMatchPacket
SupplierRefundMatchPacket
SupplierObligationMatchPacket
SupplierBalancePacket
SupplierBalanceVariancePacket
SupplierBalanceDisputePacket
UnknownInvoiceClaimPacket
DuplicateStatementClaimPacket
UnappliedCreditPacket
UnappliedPaymentPacket
CreditNoteAgingPacket
RefundAgingPacket
SupplierStatementResponsePacket
SupplierBalanceConfirmationPacket
AgedCreditorsReportPacket
APCloseCreditorPackPacket
SupplierAdminScoreUpdatePacket
SupplierStatementAuditEvidencePacket
```

Logical only. Codex maps later. No packet structs. The schema goblin can look at the packets through the window and behave.

---

## 42. What Codex Must Not Do

```text id="codex_no_supplier_recon"
Do not merge Supplier Statement Reconciliation into AP.
Do not treat supplier statements as truth.
Do not pay from supplier statement alone.
Do not overwrite AP balance with supplier statement balance.
Do not close supplier disputes without evidence.
Do not ignore missing credit notes.
Do not ignore unapplied payments.
Do not update supplier bank details from supplier statement.
Do not let GPT-5.5 accept supplier claims.
Do not let reconciliation post ledger directly.
Do not require humans to review clean matched statements.
Do not implement from this document alone.
```

---

## 43. Final Architecture Sentence

Selene Supplier Statement Reconciliation + Creditor Reporting Engine is the autonomous creditor balance-control layer that captures supplier statements, extracts statement lines, matches supplier invoice, payment, credit note, refund, adjustment, and balance claims against AP, Procurement, Receiving, Supplier Obligations, Supplier Payment, BankRec, and Accounting evidence, identifies unknown invoices, duplicate claims, missing credits, missing payments, unapplied credits, unapplied payments, refund gaps, disputed holds, and material variances, prepares supplier responses and proof packs, updates aged creditors, supplier balance confirmations, AP close packs, and supplier admin scores, and uses GPT-5.5 for human-readable explanations while deterministic Selene evidence prevents supplier statements from becoming financial truth by accident.

Simple version:

```text id="supplier_recon_simple"
Supplier sends statement.
Selene reads it.
Selene compares it to AP.
Selene checks invoices.
Selene checks credits.
Selene checks payments.
Selene checks refunds.
Selene checks receiving disputes.
Selene finds unknown or duplicate claims.
Selene sends routine responses.
Selene updates aged creditors.
Selene supports AP close.
Humans review only real unresolved exceptions.
Everything is audited.
```

That is Global Document 75 — Supplier Statement Reconciliation + Creditor Reporting Engine. Supplier statements are now no longer sacred scrolls from the vendor temple; they are claims Selene politely interrogates with AP truth, bank proof, credit notes, and the emotional patience of a robot that has seen the same invoice claimed twice.

[1]: https://cips-download.cips.org/short-reads/from-requisition-to-payment-how-the-procure-to-pay-process-streamlines-procurement?utm_source=chatgpt.com "From requisition to payment: how the procure-to-pay ..."
[2]: https://www.swift.com/standards/iso-20022/iso-20022-faqs/corporates?utm_source=chatgpt.com "ISO 20022: Corporates - Swift"
[3]: https://aurum.solutions/resources/how-to-improve-supplier-statement-reconciliation?utm_source=chatgpt.com "How to improve supplier statement reconciliation"

---

## 44. 81E B2B Statement Reconciliation + Clawback Handoff

Supplier Statement Reconciliation must account for B2B credit notes, provider payout reversals, Channel Store commission reversals, reserves, settlement holds, benefit funding adjustments, clawbacks, return courier allocations, refund adjustments, disputed commissions, and unresolved provider-fault claims created or required by 81E and Document 78.

Statements that ignore B2B holds, reversals, credit notes, clawbacks, or return courier allocations must be treated as supplier claims requiring reconciliation evidence, not AP truth.
