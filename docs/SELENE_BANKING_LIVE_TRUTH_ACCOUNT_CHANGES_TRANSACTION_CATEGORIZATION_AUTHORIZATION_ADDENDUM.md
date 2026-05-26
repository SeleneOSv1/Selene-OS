# Selene Banking Addendum — Live Bank Truth + Account Changes + Transaction Categorization + Provider Authorization + Financial Wellbeing Boundary

```text
DOCUMENT TYPE:
MASTER DESIGN ADDENDUM / BANKING + PAYMENT RAILS + RECONCILIATION + BANK ACCOUNT CHANGE GOVERNANCE + EMPLOYEE WELLBEING BOUNDARY

STATUS:
MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

APPLIES TO:
Document 5 — Selene Banking + Payment Rails + Reconciliation Master Design

PURPOSE:
Strengthen Document 5 with live/stale bank balance handling, full bank transaction feed logic, automatic bank transaction categorization, bank-native authorization, supplier/customer/company/shareholder bank account change flows, spelling/entity confirmation law, and the boundary between employer banking and employee-private financial wellbeing.
```

## 0. Authority and Scope

AGENTS.md controls execution.

This is a docs-only master design addendum.

No runtime code was changed.

This document does not authorize implementation.

Current repo truth does not prove complete runtime Banking, Provider Authorization, Bank Transaction Categorization, Employee Financial Wellbeing, or Private Vault ownership. This document defines future architecture pending Grand Architecture Reconciliation.

Future standalone engines referenced here are future design targets only and are not created by this batch.

## 1. Master Addendum Law

Selene may become the company's banking brain, but she must never become reckless with bank truth.

```text
Bank balance must show freshness.
Bank transactions must show source proof.
Bank categorization must be confidence-gated.
Bank account changes must be verified and confirmed.
Payment execution must respect bank/provider security.
Supplier, customer, employee, company, and shareholder bank changes all require protected flows.
Employee-private financial wellbeing data must not leak to employer banking or HR.
```

Selene may be brilliant. She may not become reckless with bank truth.

## 2. Live / Stale Bank Balance Law

Selene must be able to show bank balances where provider connections support it.

But she must always show whether the balance is live, recent, stale, failed, or manual.

Required fields:

```text
bank_account_id
available_balance
ledger_balance
currency
last_provider_sync_at
feed_status
balance_status
pending_outgoing_payments
pending_incoming_receipts
provider_ref
audit_ref
```

Balance statuses:

```text
live
recent
stale
manual_import
provider_failed
permission_expired
unknown
```

Selene says:

```text
Current available balance is AUD 1,790,000. Last bank sync was 9:42 AM. Pending outgoing payments total AUD 220,000.
```

If stale:

```text
The last bank update was yesterday at 4:10 PM, so I should refresh the bank feed before relying on this balance.
```

Architecture law:

```text
Banking owner records what the bank actually returned.
Selene must show "last updated" when presenting balances.
Selene must not pretend stale balances are live.
```

This preserves the requirement that Selene needs bank/API feed permissions, transaction feed, balance feed, provider proof, refresh timestamps, and failure handling.

## 3. Full Bank Transaction Read Law

Where bank/provider connections support it, Selene must read every transaction.

Transaction fields:

```text
bank_transaction_id
bank_account_id
transaction_date
value_date
amount
currency
direction: debit / credit
description
payer_name
payee_name
reference_text
bank_category_candidate
provider_category_candidate
payment_rail
provider_ref
raw_feed_ref
match_status
classification_status
audit_ref
```

Selene must support:

```text
bank deposits
bank withdrawals
supplier payments
customer receipts
payroll payments
refunds
loan repayments
lease payments
tax payments
bank fees
merchant fees
card settlements
chargebacks
interest income
interest expense
transfers
FX movements
unknown transactions
```

## 4. Automatic Bank Transaction Categorization

Selene should categorize bank transactions automatically when confidence is high.

Inputs:

```text
supplier/customer history
merchant name
bank description
amount
payment reference
invoice/bill match
receipt/invoice match
account mapping rules
tax rules
cost center rules
project refs
employee refs
cardholder refs
known recurring payment schedule
learned company pattern
approved rule pack
confidence score
```

Categorization statuses:

```text
auto_categorized
suggested_pending_review
ambiguous
manual_review_required
blocked
rejected
```

Rules:

```text
High confidence + approved rule = auto-categorize.
Low confidence = ask or route review.
Sensitive transaction = review if policy requires.
New merchant/pattern = review until learned.
```

Example:

```text
Bank transaction: Officeworks AUD 340
Known supplier: Officeworks
Usual account: Office Supplies
Receipt exists
GST valid
Confidence: 96%
Result: auto-categorized
```

Ambiguous example:

```text
Bank transaction: Transfer 5000
No reference.
Could be customer receipt, director loan, shareholder contribution, or intercompany transfer.
Result: manual review required.
```

Automatic categorization is required, but it must never become reckless guessing dressed as automation.

## 5. Bank-Native Authorization / SCA / OTP / Provider Approval Law

Selene approval is necessary, but bank-native authorization may also be required.

Correct flow:

```text
Selene prepares payment
-> Selene checks AP/Payroll/Refund/Tax/Debt source truth
-> Selene checks cashflow/reserve
-> Selene resolves approvers
-> approvers approve inside Selene with Face ID/fingerprint/passcode where required
-> Selene sends payment instruction to bank/provider
-> bank may require native OTP/SCA/banking-app approval
-> bank confirms
-> Selene records provider proof
-> Banking/Reconciliation finalizes
```

Hard law:

```text
Selene must never bypass bank-required security.
Bank-native authorization may be required even after Selene approval.
Payment is not complete until provider proof returns.
```

This is a defense-in-depth pattern: Selene internal approval first, bank/provider authorization second where required, with audit and reconciliation.

## 6. Universal Bank Account Change Governance

Selene must support bank account changes for different account owners.

Owner map:

```text
Employee salary bank account
-> Payroll owner

Supplier / vendor bank account
-> Accounts Payable / Supplier owner

Contractor bank account
-> Contractor/AP owner

Customer refund bank account
-> Accounts Receivable / Customer Refund owner

Company bank account
-> Banking / Finance owner

Shareholder dividend bank account
-> Equity / Dividends owner

Employee private personal account inside wellbeing vault
-> Employee Wellbeing / Private Vault owner
```

Banking may validate or execute payments.

Banking does not own every bank-account truth.

## 7. Universal Bank Account Change Flow

All bank-account change flows must follow the same core pattern:

```text
1. User asks Selene to change bank account.
2. PH1.D/GPT-5.5 proposes intent only.
3. PH1.N extracts owner/entity/bank fields.
4. PH1.X classifies risk.
5. Access checks scope and authority.
6. Device/Human Presence performs step-up.
7. Correct owner validates field requirements.
8. Selene confirms safe masked details before save.
9. Owner records pending/current bank account.
10. Old bank details remain historical.
11. Cutoff/effective-date/payment-hold rules apply.
12. Audit records refs only, no raw details.
```

Required step-up methods:

```text
Face ID
fingerprint
secure passcode
approved device confirmation
manager/admin fallback where policy allows
```

Important:

```text
Step-up proves stronger identity posture.
Step-up does not grant authority by itself.
```

This mirrors the employee bank account simulation, where step-up is evidence, Payroll remains owner, and confirmation is required before save.

## 8. Supplier Bank Account Change Flow

Supplier bank changes are high fraud risk.

Flow:

```text
supplier requests bank change
-> AP/Supplier owner checks supplier identity
-> Access checks requester authority
-> step-up required for internal approver
-> supplier verification required
-> old bank retained
-> new bank marked pending
-> payment hold applied if policy requires
-> first payment may require stronger approval
-> audit
```

Supplier bank change statuses:

```text
Requested
PendingSupplierVerification
PendingInternalApproval
PendingStepUp
Approved
PaymentHold
ActiveFromDate
Rejected
Cancelled
Archived
```

Selene says:

```text
This supplier bank change is sensitive. I'll verify the supplier request and hold payments to the new account until approval is complete.
```

What must not happen:

```text
no supplier bank details changed from email alone
no payment to new supplier account before verification
no old supplier bank details erased
no raw bank details in audit
```

## 9. Customer Refund Bank Account Change Flow

If a customer requests refund to a different bank account, this is high risk.

Flow:

```text
customer requests refund bank change
-> AR validates refund entitlement
-> Access checks internal approver
-> customer identity/proof checked
-> step-up for approver
-> original payment method preferred
-> different-account refund requires high-risk approval
-> Banking executes only after approval
-> audit
```

Selene says:

```text
This refund is going to a different account than the original payment method, so I need additional approval before it can be processed.
```

## 10. Company Bank Account Change Flow

Changing the company bank account is critical.

Actions include:

```text
adding new company bank account
closing bank account
changing payment-enabled status
changing payroll bank account
changing tax payment bank account
changing supplier payment source account
changing dividend payment account
```

Required controls:

```text
senior Finance authority
dual approval where policy requires
step-up
bank/provider verification
company/legal entity scope check
effective date
audit
restricted rollout
test transaction where required
```

Selene must not let a casual admin change the company's payment account because "it seemed efficient."

## 11. Shareholder / Dividend Bank Account Change Flow

Dividend bank details belong to future Equity + Dividends owner.

But Banking must support payment execution.

Flow:

```text
shareholder changes dividend account
-> Equity/Shareholder owner validates shareholder identity
-> Access checks authority/scope
-> step-up if shareholder self-service
-> bank details validated where provider exists
-> old bank retained
-> effective distribution event checked
-> audit
```

No dividend bank detail change should affect a distribution already locked unless approved.

## 12. Spelling + Entity Confirmation Law

Selene must confirm critical names and spelling before saving or paying.

Applies to:

```text
company legal name
customer legal name
supplier legal name
employee legal name
shareholder name
bank account name
account holder name
trading name
address
tax registration
invoice reference
BSB / routing / IBAN / SWIFT
account number
contact name
```

Selene must double-check when:

```text
new record
changed record
low confidence extraction
name mismatch
shortened name
abbreviation
OCR uncertainty
voice transcription uncertainty
bank account name mismatch
company name differs from trading name
```

Example:

```text
I have the supplier name as "ABC Industrial Pty Ltd." Is that the exact legal name you want on the supplier record?
```

Bank account mismatch example:

```text
The account name doesn't exactly match the supplier legal name. Should I hold this for review before saving it?
```

This extends the existing bank-account name mismatch logic in the bank-change simulation.

## 13. Employee Financial Wellbeing Boundary

This does not belong inside Document 5 as a banking feature, but Document 5 must respect it.

Create a future standalone engine:

```text
PH1.EMPLOYEE.WELLBEING
Employee Financial Wellbeing + Life Admin + Benefits Continuity Engine
```

Core model:

```text
Employer sponsors Selene.
Employee owns private financial profile.
Employer cannot see private spending, debts, loans, bills, assets, stress, or personal goals.
```

Employee Wellbeing may use:

```text
pay dates
salary
payslips
leave balances
super/pension contributions
benefits
bonus dates
expense reimbursements
salary packaging options
education support
EAP/support options
```

Employee Private Vault may contain:

```text
personal bank accounts
personal credit cards
personal loans
mortgage
rent
school fees
family expenses
personal assets
investments
insurance
personal goals
emergency savings
```

Employer may see only:

```text
anonymous aggregate wellbeing trends
benefit usage totals
program adoption
non-identifying risk trends
```

Not:

```text
Tom is behind on rent
Sarah has credit card debt
Ahmed is paying school fees late
```

That distinction matters. Otherwise Selene becomes HR surveillance instead of employee support.

## 14. Employee Lifetime Continuity

Selene must stay with the employee beyond one employer.

When employee leaves Company A:

```text
Company A access -> removed or archived by policy.
Company A work/payroll data -> locked to company scope.
Employee private vault -> remains with employee.
Employee memories/preferences -> continue under PH1.M rules.
Company A cannot see post-employment private life.
```

When employee joins Company B:

```text
Employee may connect Selene to Company B.
Company B gets only the consented employment/onboarding data.
Company B does not inherit Company A private/company records.
Employee may reuse personal profile fields where lawful and consented.
```

This becomes:

```text
Selene Personal Continuity Layer
```

owned by Employee Wellbeing + PH1.M + Identity/Access, not by Banking alone.

## 15. Consent Bridge Between Employer and Employee Private Life

Some employee-private actions may need employer systems.

Examples:

```text
salary sacrifice
super contribution change
payroll deduction
earned wage access
benefit enrollment
education assistance
expense reimbursement
financial coaching referral
EAP referral
hardship request
```

Selene must ask:

```text
Do you want me to share this request with Payroll?
```

She must not silently send private hardship, debt, loan, bill, or family-expense details to HR or Payroll.

## 16. Regulated Financial Advice Boundary

Employee wellbeing can educate, simulate, explain, and prepare requests.

But it must not become unlicensed financial advice.

Allowed:

```text
budget planning
bill reminders
debt education
pay-cycle planning
benefit explanations
super/pension contribution tracking
school fee planning
hardship letter drafting
referral to licensed adviser or EAP
```

Restricted / requires licensed pathway:

```text
personal investment advice
superannuation advice
insurance advice
retirement advice
tax advice
regulated financial product recommendation
```

Selene can educate and simulate, but financial product advice may require licensing, and Selene must not pretend to be licensed unless the right model exists.

## 17. Bank Transactions for Employee Private Vault

Employee-private bank/credit card feeds are not company bank feeds.

Document 5 company Banking owner must not mix them.

```text
Company bank feed -> company Finance/Accounting.
Employee private bank feed -> Employee Wellbeing private vault.
```

Access rule:

```text
Employer cannot view employee private feeds.
Payroll cannot view private feeds unless employee explicitly shares a specific request.
HR cannot view private feeds.
```

If employee asks:

```text
Selene, help me plan my school fees from my pay.
```

Selene uses private vault data and payroll pay-date data, but employer sees nothing.

## 18. Additions to Document 5 "What Must Not Happen"

```text
no stale balance presented as live
no bank transaction categorized by low-confidence guess
no bank payment marked complete without provider proof
no bank-required OTP/SCA/app approval bypassed
no supplier bank change from email alone
no customer refund account change without high-risk controls
no company bank account change without senior authority
no shareholder dividend bank change without shareholder/equity validation
no raw bank details in audit
no old bank details erased
no company bank feed mixed with employee private vault feed
no employer access to employee private financial life
no financial product advice without licensed/approved pathway
no spelling/name mismatch ignored on legal/payment records
```

## 19. Required Logical Packets

Future logical packets:

```text
LiveBankBalancePacket
BankFeedFreshnessPacket
BankTransactionCategorizationPacket
CategorizationConfidencePacket
BankNativeAuthorizationPacket
PaymentProviderSCAChallengePacket
UniversalBankAccountChangePacket
SupplierBankAccountChangePacket
CustomerRefundBankAccountChangePacket
CompanyBankAccountChangePacket
ShareholderDividendBankAccountChangePacket
EntitySpellingConfirmationPacket
AccountNameMismatchReviewPacket
EmployeePrivateVaultBankFeedPacket
EmployeeWellbeingConsentBridgePacket
EmployeeLifetimeContinuityPacket
FinancialAdviceBoundaryPacket
```

Codex must later map these to repo equivalents.

Do not claim they currently exist.

## 20. Example - Supplier Changes Bank Account

```text
Supplier sends new bank details.
Selene does not save from email alone.
AP opens supplier bank change review.
Supplier verification required.
Finance approver passes step-up.
Old account retained.
New account pending.
Payments held until approved.
```

Selene says:

```text
I've received new supplier bank details, but I need verification and approval before any payment can go to that account.
```

## 21. Example - Customer Refund to Different Account

```text
Customer overpaid.
Refund requested to a different bank account.
Selene flags high risk.
AR validates overpayment.
Access resolves approvers.
Approvers pass step-up.
Banking executes after approval.
Accounting posts refund.
```

Selene says:

```text
This refund is valid, but the requested account is different from the original payment method. I need extra approval before I can process it.
```

## 22. Example - Employee Leaves Company A and Joins Company B

```text
Tom leaves ABC Wines.
ABC Wines access is removed.
ABC Wines employment records remain company-scoped.
Tom's private Selene financial vault remains with Tom.
Tom later joins XYZ Logistics.
Tom consents to share only onboarding fields needed by XYZ.
Private bank, bills, school fees, loans, and goals remain hidden unless Tom chooses otherwise.
```

Selene says:

```text
Your ABC Wines company access has ended, but your personal Selene profile stays with you. If you join another employer, you can choose what details to share for onboarding.
```

## 23. Example - Company Bank Balance Is Stale

User asks:

```text
Selene, can we afford this payment?
```

Bank feed:

```text
last sync yesterday
pending payments changed today
```

Selene says:

```text
I need to refresh the bank feed first. The last balance is from yesterday, so I shouldn't rely on it for a payment decision.
```

## 24. Example - Automatic Categorization

Bank transaction:

```text
Officeworks AUD 340
```

Evidence:

```text
known supplier
office supplies account mapping
receipt attached
GST valid
confidence 96%
```

Result:

```text
auto-categorize
```

Ambiguous case:

```text
Bank transaction:
Transfer AUD 5,000

Evidence:
no reference
two possible customer invoices
possible director loan

Result:
manual review
```

Selene says:

```text
I can't safely categorize this transfer yet. It could be customer payment or director funding. I've opened a review.
```

## 25. Future Simulation Targets

```text
SIM_BANK_016_live_balance_stale_feed_warning
SIM_BANK_017_bank_transaction_auto_categorized_high_confidence
SIM_BANK_018_bank_transaction_low_confidence_review
SIM_BANK_019_supplier_bank_account_change_verification
SIM_BANK_020_customer_refund_bank_account_high_risk
SIM_BANK_021_company_bank_account_change_dual_authority
SIM_BANK_022_bank_native_sca_required_after_selene_approval
SIM_BANK_023_shareholder_dividend_bank_change
SIM_EMP_WELLBEING_001_employee_private_vault_created
SIM_EMP_WELLBEING_002_employee_leaves_company_private_profile_continues
SIM_EMP_WELLBEING_003_employee_joins_new_employer_consent_bridge
SIM_EMP_WELLBEING_004_private_bill_planning_not_visible_to_employer
SIM_EMP_WELLBEING_005_salary_sacrifice_request_requires_consent_bridge
```

## 26. Final Addendum Architecture Sentence

Selene Banking + Payment Rails must know live bank truth without pretending stale feeds are current, categorize transactions automatically only when evidence and confidence are strong, require bank-native provider authorization where banks demand it, govern supplier/customer/company/shareholder bank-account changes through step-up, confirmation, owner validation, history retention, and audit, and keep employee-private financial wellbeing data completely separate from employer banking while still allowing consent-based bridges for benefits, payroll deductions, salary sacrifice, and approved support programs.
