# Selene Credit Cards + Employee Spend + Reimbursements Master Design

```text
DOCUMENT TYPE:
MASTER DESIGN / CREDIT CARDS + EMPLOYEE SPEND + EXPENSE CLAIMS + REIMBURSEMENTS + PERSONAL CHARGE RECOVERY

STATUS:
MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

PURPOSE:
Define Selene's company card, executive card, employee spend, receipt capture, personal charge detection, reimbursement, salary advance/repayment interaction, spend budget, credit card feed, card reconciliation, tax/claimability, AP/Payroll/Accounting handoff, and offboarding card-control system.
```

## 0. Authority and Scope

AGENTS.md controls execution.

This is a docs-only master design.

No runtime code was changed.

This document does not authorize implementation.

Current repo truth does not prove complete runtime Credit Cards, Employee Spend, Reimbursement, Card Provider, Payroll Deduction, Fleet, Insurance, or Employee Wellbeing ownership. This document defines future architecture pending Grand Architecture Reconciliation.

Future implementation requires repo-truth activation, approved file scope, tests, backend evidence, provider-off/fake-provider proof, Access/Authority proof, audit proof, and JD approval.

## 1. Executive Target

Selene must manage company cards and employee spending automatically.

Old way:

```text
employee uses company card
receipt disappears
manager forgets
finance chases everyone
personal charge hides in the statement
card statement arrives
human reconciles it later
everyone lies to themselves and calls it a process
```

Selene way:

```text
card transaction appears
Selene identifies cardholder, merchant, category, project, cost center, and likely business purpose
Selene asks for receipt if missing
Selene detects personal or suspicious spend
Selene checks spend budget and card policy
Selene routes approval where needed
Selene recovers personal charges if policy/law allows
Selene posts the correct accounting evidence
Selene reconciles card statement automatically
```

The goal:

```text
no missing card receipts
no hidden executive personal charges
no unapproved employee spend
no personal charges treated as business expenses
no reimbursements paid twice
no credit card statement left unreconciled
no employee card left active after termination
```

Tiny miracle: the company card stops being a magical plastic rectangle of mystery.

## 2. Master Law

```text
Every card transaction must be classified.

Every employee spend item must have business purpose or personal classification.

Every personal/non-business charge must be recovered or reclassified through policy.

Every reimbursement must trace to receipt, approval, payment, and accounting evidence.

Every card transaction must reconcile to provider/bank feed.

Every sensitive spend action must respect Access, Authority, Simulation, PH1.WRITE, and Audit.

PH1.D/GPT-5.5 may help classify and explain.

PH1.D/GPT-5.5 must not approve spend, deduct salary, hide personal spend, invent tax claimability, or post final accounting truth.
```

## 3. Owner Split

### Credit Cards / Employee Spend Owns

```text
company card assignment
executive card assignment
employee card limits
employee spend budgets
merchant/category policy
receipt collection
business-purpose collection
personal vs business classification
missing receipt workflow
spend exception workflow
cardholder accountability
reimbursement claim lifecycle
employee advance interaction
personal charge recovery request
card offboarding readiness
card spend audit evidence
```

### Banking / Payment Rails Owns

```text
credit card feed
card statement feed
card payment proof
merchant settlement evidence
card fees
chargebacks
refund settlement evidence
card reconciliation proof
provider/bank confirmation
```

### Accounting Owns

```text
expense journal
asset/inventory/prepayment journal where applicable
card liability journal
bank/card payment journal
personal spend receivable journal
director/shareholder loan posting where applicable
reimbursement posting
fee posting
tax/GST/VAT posting
```

### Payroll Owns

```text
salary deduction where lawful and approved
employee advance repayment
payroll reimbursement where company policy routes reimbursement through payroll
employee-visible deduction explanation
final-pay recovery where lawful
```

### Accounts Payable Owns

```text
supplier reimbursement/payment route where employee spend becomes supplier/AP obligation
card provider bill/payment if handled as AP
```

### Tax / GST / VAT Owns

```text
claimability
tax code
business/private use split
receipt/tax invoice validity
country/region deduction rules
non-claimable treatment
```

### Finance / Budget Owns

```text
department spend budget
employee spend budget
executive spend policy
card limit approval
budget variance
cashflow impact
spend control
```

### Access / Governance Owns

```text
who can issue card
who can view card transactions
who can view executive card spend
who can approve expense
who can approve personal charge recovery
who can approve payroll deduction
who can approve reimbursement
who can change card limit
who can suspend card
who can override missing receipt
who can export card reports
```

### HR / Offboarding Owns

```text
employment status
resignation/termination
card return checklist
employee lifecycle event
final-pay/offboarding context
```

### PH1.REM Owns

```text
missing receipt reminders
cardholder follow-up timing
approval reminder timing
reimbursement follow-up
offboarding card return reminders
```

### PH1.BCAST / PH1.DELIVERY Owns

```text
cardholder notifications
manager approval requests
missing receipt notices
personal charge explanation delivery
reimbursement confirmations
card suspension notices
```

### PH1.WRITE Owns

```text
employee-facing card messages
manager approval summaries
personal charge recovery explanations
missing receipt wording
reimbursement wording
policy denial wording
```

### PH1.D / GPT-5.5 May Assist

```text
merchant description interpretation
business-purpose wording
receipt summary
expense category suggestion
personal-spend likelihood proposal
employee-friendly explanation
manager approval summary
policy explanation
```

But must not:

```text
approve card spend
deduct salary
decide final tax claimability
decide final business/personal truth
hide transaction
post final journal
override card policy
```

## 4. Card Types

Selene must support multiple card types.

```text
company credit card
executive credit card
employee credit card
department card
fuel card
travel card
purchase card
virtual card
single-use card
subscription card
merchant-specific card
project card
fleet card
```

Each card may have different rules.

### Card Profile Fields

```text
card_id
tenant_id
company_id
legal_entity_id
card_provider_ref
card_type
cardholder_user_id
assigned_department_id
assigned_cost_center_id
currency
card_limit
period_limit
single_transaction_limit
merchant_category_rules
allowed_categories
blocked_categories
receipt_required
business_purpose_required
approval_policy_ref
status
issued_at
expires_at
audit_ref
```

### Card Statuses

```text
Draft
PendingApproval
Active
Suspended
Lost
Stolen
Cancelled
Expired
OffboardingHold
Archived
```

## 5. Card Assignment

Before a card is issued, Selene must know:

```text
who receives the card
why they need it
what limit applies
which department/cost center pays
which merchant categories are allowed
whether receipts are mandatory
who approves exceptions
what happens at resignation/termination
```

Card assignment flow:

```text
manager requests card
-> Access checks requester authority
-> Finance checks limit/budget
-> HR confirms employment status
-> card policy selected
-> cardholder accepts terms
-> card issued or provider request prepared
-> audit
```

Selene says:

```text
I can prepare the company card for Tom. I'll set it to the Warehouse cost center, monthly limit AUD 2,000, receipts required, and block personal categories unless Finance approves otherwise.
```

## 6. Card Transaction Feed

Banking/Card Provider supplies transaction evidence.

### Card Transaction Fields

```text
card_transaction_id
card_id
cardholder_user_id
merchant_name
merchant_category_code
transaction_date
posted_date
amount
currency
country
description
provider_ref
receipt_required
receipt_status
business_purpose_status
classification_status
reconciliation_status
audit_ref
```

Card transaction statuses:

```text
Received
PendingReceipt
PendingBusinessPurpose
PendingClassification
PendingApproval
ApprovedBusiness
ClassifiedPersonal
Disputed
Rejected
Reimbursable
Recovered
Posted
Reconciled
Archived
```

Banking owns feed proof.

Employee Spend owns classification workflow.

Accounting owns posting.

## 7. Automatic Transaction Classification

Selene should classify card transactions automatically when confidence is high.

Inputs:

```text
merchant name
merchant category
cardholder
department
cost center
usual spend pattern
amount
receipt text
invoice/tax details
project/task context
travel calendar
fleet/vehicle context
approved supplier list
company policy
tax claimability rules
prior classifications
```

Classification options:

```text
business_expense
personal_charge
mixed_use
requires_receipt
requires_manager_review
requires_tax_review
requires_policy_review
fraud_suspected
duplicate_suspected
```

Confidence rules:

```text
high confidence + approved policy = auto-classify
medium confidence = ask cardholder or manager
low confidence = review required
sensitive category = review required
new merchant = review until learned
```

Example:

```text
Card transaction: BP Fuel AUD 110
Cardholder: Tom
Assigned vehicle: Fleet Van 12
Receipt uploaded
Fleet confirms vehicle use
Result: business fuel expense
```

Example:

```text
Card transaction: Luxury Spa AUD 480
Cardholder: Harry CEO
No business purpose
Result: likely personal charge, review required
```

Yes, even executives. Especially executives. Review discipline applies equally.

## 8. Receipt Capture

If receipt is required, Selene must chase it.

Receipt sources:

```text
camera photo
PDF upload
email forward
merchant e-receipt
POS receipt
supplier invoice
card provider enhanced data
```

Receipt fields:

```text
receipt_id
card_transaction_id
uploaded_by_user_id
upload_method
merchant_name_candidate
amount_candidate
tax_amount_candidate
date_candidate
currency_candidate
scan_status
tax_invoice_valid_candidate
confidence_summary
artifact_ref
audit_ref
```

If missing:

```text
PH1.REM schedules reminder.
PH1.WRITE writes human prompt.
PH1.BCAST/DELIVERY notifies cardholder.
```

Selene says:

```text
I need the receipt for the AUD 110 fuel transaction. You can take a photo now or upload it later.
```

If ignored:

```text
missing_receipt_escalation_policy applies
```

Possible outcomes:

```text
expense held
cardholder reminded
manager notified
personal recovery candidate
tax claim blocked
card suspended if repeated
```

## 9. Business Purpose

Some expenses need business purpose.

Selene asks simply:

```text
What was this purchase for?
```

Cardholder says:

```text
Fuel for the North Warehouse delivery van.
```

PH1.N extracts:

```text
purpose: fuel
vehicle_ref: Fleet Van 12
location: North Warehouse
confidence: medium_high
```

Deterministic owners verify:

```text
Fleet confirms vehicle context where available.
Tax checks claimability.
Accounting maps account.
```

GPT-5.5 may help phrase and summarize.

It does not decide final claimability.

## 10. Personal Charge Detection

Selene must detect likely personal charges.

Signals:

```text
merchant category personal
merchant not allowed
weekend/after-hours unusual
location unrelated to work
no receipt
cardholder says personal
manager marks personal
tax/private-use rule says personal portion
merchant description indicates personal use
same pattern repeated
```

Personal charge outcomes:

```text
employee repayment request
payroll deduction where lawful/approved
director/shareholder loan account
expense rejection
mixed-use split
manager review
Finance review
card policy warning
card suspension if repeated
```

Important:

```text
Personal does not always mean fraud.
It may be honest mistake.
Selene should clarify before accusing.
```

Selene says:

```text
This looks like it may be personal rather than business. Was this for work, or should I mark it for repayment?
```

Human, direct, not robotic.

## 11. Executive Card Personal Spend

Executive cards need strict review.

Executives may have broader card permissions, but not magical immunity.

Selene must review:

```text
meals
travel
entertainment
gifts
fuel
personal items
family expenses
luxury spend
weekend spend
cash withdrawals
unusual merchant categories
```

Recovery options:

```text
payroll deduction where lawful
executive repayment
director loan account
shareholder loan account
expense reclassification
board/finance review
taxable benefit/fringe benefit treatment where applicable
```

Example:

```text
Harry CEO spends AUD 480 on a personal dinner.
Selene asks for business purpose.
Harry marks personal.
Selene creates recovery path:
  - payroll deduction if lawful/approved, or
  - director loan account, or
  - direct repayment.
Accounting receives correct handoff.
```

Selene says:

```text
Thanks for confirming. I'll mark this as personal and route it through the approved recovery method, so it doesn't sit in business expenses.
```

## 12. Mixed-Use Expenses

Some transactions are partly business, partly personal.

Examples:

```text
phone bill
internet
vehicle fuel
travel
accommodation
meals
software subscription
home office
```

Fields:

```text
business_percentage
personal_percentage
basis_of_split
approved_by
tax_rule_ref
payroll_recovery_ref
accounting_ref
audit_ref
```

Example:

```text
Phone bill AUD 100
Business use 70%
Personal use 30%

Business expense = AUD 70
Personal recovery = AUD 30
```

Tax owner validates claimability.

Accounting posts split.

Payroll/AP handles recovery if needed.

## 13. Employee Reimbursements

Not all employee spend uses company card.

Employee may pay personally and ask for reimbursement.

Reimbursement sources:

```text
receipt upload
mileage claim
travel expense
meal expense
software purchase
work supplies
client entertainment
training cost
fuel
parking/tolls
```

Reimbursement flow:

```text
employee submits claim
-> Selene captures receipt/evidence
-> business purpose collected
-> policy checked
-> tax claimability checked
-> budget checked if needed
-> manager/Finance approval if required
-> reimbursement payment path selected
-> Payroll/AP/Banking executes depending policy
-> Accounting posts
-> employee notified
```

Reimbursement states:

```text
Draft
Submitted
PendingReceipt
PendingManagerApproval
PendingFinanceApproval
Approved
Rejected
ScheduledForPayment
Paid
Reconciled
Archived
```

Selene says:

```text
I've got the receipt. This looks like a warehouse supply purchase. I'll send it for approval and let you know when reimbursement is scheduled.
```

## 14. Expense Claim Fields

```text
expense_claim_id
employee_id
company_id
claim_type
merchant_name
transaction_date
amount
currency
receipt_ref
business_purpose
cost_center_ref
project_ref
tax_code_candidate
reimbursement_method
approval_status
payment_status
accounting_status
audit_ref
```

Expense claim types:

```text
travel
meal
fuel
parking
toll
office_supplies
training
software
client_entertainment
home_office
phone_internet
mileage
other
```

## 15. Mileage and Travel Claims

Selene must support mileage/travel claims where policy allows.

Mileage fields:

```text
trip_date
start_location
end_location
distance
vehicle_type
business_purpose
rate_per_km_or_mile
claim_amount
policy_ref
approval_ref
audit_ref
```

Travel fields:

```text
travel_date
destination
customer/project_ref
transport_type
hotel_ref
meal_ref
per_diem_ref
business_purpose
approval_ref
```

Country/tax rules may apply.

No GPT-5.5 invented reimbursement rate.

## 16. Employee Advances

Employees may receive money in advance.

Examples:

```text
salary advance
travel advance
expense advance
cash advance
project advance
```

Advance fields:

```text
advance_id
employee_id
amount
currency
advance_type
issued_date
repayment_schedule_ref
remaining_balance
deduction_method
approval_ref
status
audit_ref
```

Repayment options:

```text
payroll deduction where lawful
expense offset
direct employee repayment
final pay recovery where lawful
installment plan
```

If employee asks to delay repayment:

```text
Selene checks policy
payroll cutoff
approval requirement
remaining balance
cash impact
employee explanation
```

Selene says:

```text
I can request to move this deduction to next payrun. It needs payroll approval because the repayment schedule is already active.
```

Hard rule:

```text
No hidden deduction from salary.
Employee must see the repayment schedule and deduction reason.
```

## 17. Personal Charge Recovery

Personal/non-business charges must be recovered according to policy.

Recovery methods:

```text
payroll deduction where lawful and approved
direct repayment by employee
deduct from reimbursement owed
director/shareholder loan account
expense rejection
final pay recovery where lawful
```

Recovery fields:

```text
recovery_id
employee_id
transaction_id
amount
currency
reason
recovery_method
approval_required
employee_acknowledgement_ref
payroll_deduction_ref
accounting_ref
status
audit_ref
```

Recovery statuses:

```text
Draft
PendingEmployeeResponse
PendingApproval
Approved
Scheduled
Recovered
Disputed
Escalated
Cancelled
Archived
```

If employee disputes:

```text
Selene opens EmployeeSpendDisputePacket
collects explanation/evidence
routes to manager/Finance
holds recovery until resolved if policy requires
```

## 18. Payroll Deduction Boundary

Payroll deductions are sensitive.

Employee Spend may request recovery.

Payroll owns deduction execution.

Flow:

```text
personal charge confirmed
-> recovery method = payroll deduction
-> Access/Policy checks legality/authority
-> employee acknowledgement where required
-> Payroll receives deduction request
-> Payroll applies in payrun if allowed
-> Payslip explains deduction
-> Accounting records recovery
```

Payroll deduction must not be hidden.

Selene says:

```text
This personal charge will be recovered from your next payrun, subject to payroll rules. It will appear clearly on your payslip.
```

If law/policy forbids deduction:

```text
use direct repayment or receivable instead
```

## 19. Card Spend Budgets

Finance may set budgets for cards.

Budget dimensions:

```text
employee_id
executive_id
department_id
cost_center_id
project_id
card_id
merchant_category
monthly_limit
single_transaction_limit
travel_limit
fuel_limit
entertainment_limit
approval_threshold
effective_dates
```

Budget checks:

```text
within_budget
near_budget
over_budget
requires_approval
blocked
```

Selene says:

```text
This purchase would take the Marketing card over its monthly travel budget. I'll request approval before it can be accepted as company spend.
```

## 20. Merchant Category Controls

Cards may have merchant controls.

```text
allowed_merchants
blocked_merchants
allowed_mcc_codes
blocked_mcc_codes
country_restrictions
online_purchase_allowed
cash_withdrawal_allowed
subscription_allowed
fuel_only
travel_only
single_use
```

If transaction violates control:

```text
flag exception
ask cardholder
route review
possible card suspension
```

## 21. Missing Receipt Policy

Companies need policy for missing receipts.

Options:

```text
allow under threshold with declaration
manager approval required
tax claim blocked
personal recovery if no proof
card suspended after repeated missing receipts
reimbursement denied
```

Missing receipt declaration fields:

```text
declaration_id
employee_id
transaction_id
reason
business_purpose
manager_approval_ref
tax_claim_allowed
audit_ref
```

Selene says:

```text
If you can't find the receipt, I can prepare a missing-receipt declaration. Finance may still need to approve it.
```

## 22. Credit Card Reconciliation

Banking provides card statement/feed.

Employee Spend classifies transactions.

Accounting posts.

Reconciliation checks:

```text
card transaction exists
receipt/business purpose complete
classification complete
approval complete
tax treatment complete
personal recovery handled
accounting entry posted
card provider statement matched
payment to card provider matched
```

Reconciliation statuses:

```text
Unmatched
PendingReceipt
PendingClassification
PendingApproval
PendingRecovery
ReadyForPosting
Posted
Reconciled
Exception
Archived
```

## 23. Card Provider Fees and Disputes

Document 5 owns provider fee evidence and chargebacks.

Document 6 consumes cardholder context.

Examples:

```text
card annual fee
foreign transaction fee
cash advance fee
late fee
chargeback
merchant refund
transaction reversal
```

If abnormal:

```text
Banking creates fee exception.
Employee Spend checks cardholder context.
Accounting posts after review.
```

## 24. Fraud, Lost, or Stolen Card

Selene must support card security events.

Flow:

```text
cardholder reports lost/stolen
-> Selene identifies card
-> Access/Policy validates user
-> Banking/Card Provider suspension route
-> recent transactions reviewed
-> suspicious transactions flagged
-> replacement card request if allowed
-> audit
```

Selene says:

```text
I'll lock this card now and flag recent transactions for review. If you see anything you don't recognize, tell me which one.
```

If live provider not available, Selene guides immediate bank/card provider contact and records incident.

## 25. Card Offboarding

When employee resigns, is terminated, or changes role:

```text
HR event triggers card review
-> Employee Spend checks assigned cards
-> card return/cancellation required
-> pending receipts chased
-> personal charges resolved
-> advances/recoveries reviewed
-> final pay recovery if lawful
-> card deactivated
-> audit
```

Selene says:

```text
Before offboarding is complete, I need Tom's company card returned and two missing receipts resolved.
```

No employee card should remain active after termination unless explicitly authorized for a special transition.

## 26. Executive Offboarding

Executive cards require extra review.

```text
open card charges
personal charges
director/shareholder loan balance
travel advances
unsubmitted receipts
subscriptions linked to card
pending reimbursements
board approval items
```

Executive card offboarding may need Finance/Board review.

## 27. Subscriptions on Cards

Selene must detect recurring subscriptions charged to cards.

Fields:

```text
subscription_id_candidate
merchant_name
frequency
amount
currency
card_id
owner_user_id
business_owner_id
renewal_date
cancellation_policy
accounting_category
approval_status
```

Selene should ask:

```text
This looks like a recurring subscription. Who owns it, and should it continue?
```

If employee leaves:

```text
reassign subscription owner
change payment method
cancel subscription
```

## 28. Integration With Fleet

Fleet-related card charges may include:

```text
fuel
charging
tolls
parking
maintenance
repairs
insurance
registration
roadside assistance
```

Document 6 must hand these to future PH1.FLEET where relevant.

Flow:

```text
card transaction received
-> merchant/category suggests fleet
-> vehicle/driver context requested
-> Fleet validates vehicle/usage if available
-> Accounting maps fuel/repair/toll/parking
```

Fleet owns vehicle truth.

Employee Spend owns card transaction workflow.

## 29. Integration With Benefits / Employee Wellbeing

Some employee spend may relate to benefits:

```text
education reimbursement
wellbeing allowance
health benefit
work-from-home allowance
tool allowance
uniform allowance
travel benefit
salary packaging
```

This must connect to future:

```text
PH1.EMPLOYEE.WELLBEING
```

But private employee life data must remain separate.

Employer sees only benefit claim/request data, not private personal finance details.

## 30. Tax / GST / VAT Claimability

Selene must determine whether card spend/reimbursement is claimable.

Inputs:

```text
receipt/tax invoice
merchant
category
business purpose
country/region
private/business split
employee role
asset/expense classification
tax rule pack
```

Tax outcomes:

```text
fully_claimable
partially_claimable
not_claimable
requires_tax_review
```

Selene must not claim tax without valid evidence.

PH1.D can explain.

Tax owner decides.

## 31. Access and Authority

Protected actions:

```text
issue company card
change card limit
approve expense
approve missing receipt
approve personal charge recovery
approve payroll deduction
approve reimbursement
approve executive spend
write off employee spend
suspend card
reactivate card
export card report
view executive card spend
view employee personal recovery details
```

Authority depends on:

```text
amount
employee role
card type
merchant category
department
budget
country
tax risk
personal charge risk
executive status
repeat behavior
```

Step-up may be required for:

```text
card limit increase
large reimbursement
payroll deduction approval
executive personal charge recovery
card suspension/reactivation
export sensitive card report
```

## 32. PH1.D / GPT-5.5 Role

Allowed:

```text
suggest expense category
summarize receipt
draft employee message
draft manager approval summary
explain why receipt is needed
explain personal charge recovery
help classify merchant description
translate receipt text
summarize card exceptions
```

Forbidden:

```text
approve expense
approve reimbursement
approve payroll deduction
decide tax claimability
final classify personal/business in disputed case
hide executive spend
post journal
reconcile card as final truth
```

## 33. PH1.WRITE Wording

PH1.WRITE owns final wording.

### Missing Receipt

```text
I need the receipt for the AUD 110 fuel charge so Finance can finish the card review. You can take a photo now or upload it later.
```

### Possible Personal Charge

```text
This transaction may be personal rather than business. Was it for work, or should I mark it for repayment?
```

### Reimbursement Approved

```text
Your reimbursement has been approved and scheduled for payment. I'll let you know when it is processed.
```

### Deduction Explanation

```text
This personal card charge will be recovered according to company policy. It will be shown clearly on your payslip if payroll deduction is used.
```

## 34. Audit Requirements

Audit must record:

```text
audit_event_id
actor_id
action
card_id
transaction_id
employee_id
merchant_ref
amount
currency
receipt_ref
classification_old_ref
classification_new_ref
business_purpose_ref
approval_refs
step_up_refs
tax_rule_ref
recovery_ref
payroll_deduction_ref
accounting_ref
banking_feed_ref
timestamp
company_id
legal_entity_id
country
reason_code
```

No raw card numbers.

No hidden executive exceptions.

No deleted spend history.

## 35. Failure Branches

### Missing Receipt

```text
transaction remains pending
tax claim blocked or limited
employee reminded
manager/Finance escalated if overdue
```

### Employee Says Charge Was Business But No Proof

```text
request explanation/evidence
manager review
tax claim may be blocked
```

### Employee Confirms Personal Charge

```text
create recovery flow
choose recovery method
audit
```

### Employee Disputes Personal Classification

```text
open spend dispute case
collect evidence
route to manager/Finance
hold recovery if policy requires
```

### Card Feed Fails

```text
Banking creates provider exception
reconciliation delayed
card transactions not final
```

### Cardholder Terminated

```text
card suspended/cancelled
open transactions reviewed
missing receipts chased
personal charges recovered where lawful
```

### Duplicate Reimbursement

```text
block payment
show suspected duplicate claim
review required
```

## 36. Required Logical Packets

Future logical packets:

```text
CompanyCardPacket
CardAssignmentPacket
CardPolicyPacket
CardTransactionPacket
CardTransactionClassificationPacket
ReceiptCapturePacket
BusinessPurposePacket
PersonalChargeReviewPacket
PersonalChargeRecoveryPacket
EmployeeReimbursementClaimPacket
ExpenseApprovalPacket
MissingReceiptDeclarationPacket
CardSpendBudgetPacket
MerchantCategoryRulePacket
EmployeeAdvancePacket
PayrollDeductionRequestPacket
CardReconciliationPacket
CardOffboardingPacket
CardFraudIncidentPacket
SubscriptionOnCardPacket
FleetCardSpendHandoffPacket
CardTaxClaimabilityPacket
EmployeeSpendAuditEvidencePacket
```

Codex must later map these to repo equivalents.

Do not claim they currently exist.

## 37. Example - Executive Personal Charge

```text
Harry CEO uses company card for AUD 480 dinner.
Selene requests business purpose.
Harry says it was personal.
Selene creates personal charge recovery.
Policy routes to director loan account, not business meal expense.
Accounting receives handoff.
Audit records evidence.
```

Selene says:

```text
Thanks for confirming. I'll mark this as personal and route it through the approved recovery method so it doesn't remain in business expenses.
```

## 38. Example - Employee Fuel Receipt

```text
Tom buys fuel for AUD 110.
Card feed arrives.
Selene asks for receipt.
Tom uploads receipt.
Fleet confirms vehicle use.
Tax owner validates claimability.
Accounting maps to vehicle fuel expense.
Card transaction reconciles.
```

Selene says:

```text
That fuel receipt is matched to the warehouse delivery van. I've classified it as business fuel and sent the evidence for accounting.
```

## 39. Example - Missing Receipt Repeated

```text
Sarah has 4 missing receipts this month.
Policy says 3 missing receipts triggers manager review.
Selene sends manager summary.
Card may be restricted until resolved.
```

Selene says:

```text
Sarah has four card transactions still missing receipts. This now needs manager review under the card policy.
```

## 40. Example - Reimbursement

```text
Ahmed buys safety gloves with personal card.
Uploads receipt.
Selene checks policy.
Manager approves.
Finance approves if threshold requires.
Reimbursement scheduled.
Payment confirmed.
Accounting posts expense and reimbursement.
```

Selene says:

```text
Your safety-glove reimbursement has been approved and scheduled. I'll confirm once the payment is processed.
```

## 41. What Must Not Happen

```text
no card transaction ignored
no missing receipt forgotten
no personal charge hidden as business expense
no executive exception hidden
no salary deduction without policy/legal/approval path
no old card transaction deleted
no card feed treated as reconciled without provider proof
no reimbursement paid twice
no tax claim without valid evidence
no cardholder allowed active card after termination unless explicitly approved
no GPT-5.5 final approval or classification in disputed cases
no Accounting posting without evidence
no Banking/Card feed mixed with employee private financial vault
no implementation from this document alone
```

## 42. Future Simulation Targets

```text
SIM_CARD_001_card_transaction_auto_classified_high_confidence
SIM_CARD_002_missing_receipt_reminder_and_escalation
SIM_CARD_003_executive_personal_charge_recovery
SIM_CARD_004_employee_personal_charge_payroll_deduction_request
SIM_CARD_005_employee_reimbursement_approval_and_payment
SIM_CARD_006_duplicate_reimbursement_blocked
SIM_CARD_007_cardholder_termination_card_offboarding
SIM_CARD_008_lost_card_suspension_and_recent_transaction_review
SIM_CARD_009_mixed_use_phone_bill_split
SIM_CARD_010_fleet_fuel_card_transaction_handoff
SIM_CARD_011_subscription_on_departing_employee_card
SIM_CARD_012_abnormal_card_fee_review
```

## 43. Related Addendum

Immediate card stop rules, role-change card review, card limit intelligence, card issuance intelligence, instant receipt capture, receipt capture before feed match, weekly/monthly card review cadence, all-transaction evidence discipline, role-based card templates, and insurance/fleet boundaries are defined in SELENE_CREDIT_CARDS_CARD_LIFECYCLE_INSTANT_RECEIPT_CAPTURE_LIMIT_INTELLIGENCE_ADDENDUM.md and must be read with this document.

## 44. Final Architecture Sentence

Selene Credit Cards + Employee Spend + Reimbursements is the governed employee-spend control system: it receives company and executive card feeds, captures receipts, classifies transactions, detects personal and mixed-use charges, enforces card budgets and merchant policies, manages reimbursements and employee advances, recovers personal spend through lawful approved methods, connects fleet and benefits where relevant, reconciles card statements through Banking, posts accounting only from evidence, protects tax claimability, and ensures Selene can manage company spending automatically without letting plastic cards become unmanaged financial disasters.
