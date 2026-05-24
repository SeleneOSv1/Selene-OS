# Selene Payroll/HR Evidence Fabric + Codex Readiness Conversion Layer

MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

## 0. Authority and Scope

AGENTS.md controls execution.

This is a docs-only readiness/conversion layer.

No implementation, runtime mutation, migration, packet/schema implementation, provider implementation, Payroll implementation, HR implementation, Compensation implementation, Finance/Accounting implementation, Bank provider implementation, Scheduler/Roster/Attendance implementation, Task/HWM implementation, Access implementation, Onboarding implementation, Reminder implementation, Broadcast/Delivery implementation, PH1.D implementation, PH1.N implementation, PH1.X implementation, PH1.WRITE implementation, Desktop/iPhone edit, Adapter edit, old-path deletion, or cleanup is authorized by this document.

This document preserves all five Payroll/HR automation master designs and the 43-point revision list. It does not replace the five master designs. It makes them buildable later by defining the missing mechanical layer.

## 1. Why This Layer Exists

The five Payroll/HR automation blocks are strong business architecture.

The 43-point revision identifies missing implementation detail.

The Evidence Fabric document adds:

- owner map,
- evidence packets,
- trust model,
- state machines,
- simulations,
- authority matrix,
- failure modes,
- audit model,
- privacy model,
- staged build order,
- test matrix.

It also converts the correction "Selene helps humans prepare payroll" into "Selene continuously manages evidence, exceptions, disputes, and payment handoff while canonical engines own truth."

## 2. Canonical Engine Owner Map

| Future Owner | Owns | Does Not Own |
|---|---|---|
| HR | employment profile, employment status, recruitment, candidate records, offer context, probation, resignation, termination, retirement, offboarding, rehire, HR notices | pay calculation, roster truth, task truth, bank rails |
| Payroll | payroll profile, payrun, payslip, payroll draft, payroll correction, gross/net pay calculation, deductions, contributions, final pay, employee pay instruction | employment lifecycle, contractor AP payment by default, bank transfer execution |
| Compensation | salary/rate policy, overtime multipliers, weekend/holiday rates, allowances, bonus/commission rules, approved pay overrides | payrun execution, sales truth, money movement |
| Leave/Benefits/Final Pay | leave policy, leave balance, benefit rules, final-pay evidence, payout policy | attendance truth, bank rails |
| Contractor/AP | contractor billing mode, contractor evidence, invoice approval, AP payment instruction | employee payroll by default |
| Finance/Accounting | ledger validation, payment files, journals, accounting entries, budget/money approval, reconciliation | employee pay truth, HR truth |
| Bank/Payment Provider | transfer execution, confirmation, provider failure evidence | payroll/HR decisioning |
| Scheduler/Roster | planned work, shifts, roster, future shift removal | actual worked time, pay calculation |
| Attendance/Timesheet | actual worked time, breaks, clock-in/out, location proof, timesheets, approved overtime evidence | pay calculation |
| Task/HWM | work-output evidence, handover, task/milestone completion, performance evidence | payroll calculation, HR discipline |
| Access/Governance | field-level view/edit/approve, protected-action gates, step-up requirements, authority matrix | payroll/HR source truth |
| PH1.D | GPT-5.5 proposal/help only | final payroll/HR truth, execution |
| PH1.N | extraction candidates | authority or execution |
| PH1.X | route/risk validation | domain mutation |
| PH1.WRITE | final human explanation | payroll/HR calculation or state |
| PH1.REM | reminder timing | source truth or delivery |
| PH1.BCAST/DELIVERY | message delivery and delivery attempt truth | payroll/HR source truth |
| Audit | immutable evidence/provenance records | business calculation or policy decisions |

Selene Orchestration coordinates. It does not own payroll truth, HR truth, roster truth, task truth, payment truth, or contractor/AP truth.

## 3. Payroll Evidence Fabric

Evidence Fabric is the canonical bridge. Everything that affects pay must enter Payroll through evidence packets.

Required future packets:

- `PayrollEvidenceBundlePacket`
- `AttendanceEvidencePacket`
- `RosterEvidencePacket`
- `LocationProofPacket`
- `TaskCompletionEvidencePacket`
- `TimesheetEvidencePacket`
- `OvertimeApprovalPacket`
- `LeaveEvidencePacket`
- `HolidayEvidencePacket`
- `CommissionEvidencePacket`
- `BonusEvidencePacket`
- `DeductionEvidencePacket`
- `AdvanceRepaymentPacket`
- `BankAccountEvidencePacket`
- `TaxContributionEvidencePacket`
- `BenefitsEvidencePacket`
- `ContractorEvidencePacket`
- `AccountingHandoffPacket`
- `PaymentInstructionPacket`
- `PaymentConfirmationPacket`
- `PayrollExceptionPacket`

Payroll calculates from evidence bundles, not raw conversation.

## 4. PayrollEvidenceBundlePacket

Future logical packet fields:

| Field | Purpose |
|---|---|
| `bundle_id` | stable evidence bundle id |
| `employee_id` | payroll subject |
| `pay_period` | pay period covered |
| `tenant_id` | tenant scope |
| `company_id` | company/legal entity scope |
| `jurisdiction` | country/region/legal rule scope |
| `payroll_group` | payroll group/frequency/ref |
| `employment_status_ref` | HR employment state reference |
| `pay_profile_ref` | Payroll pay profile reference |
| `roster_evidence_refs` | planned work refs |
| `attendance_evidence_refs` | actual worked-time refs |
| `task_evidence_refs` | task/milestone/handover evidence refs |
| `leave_evidence_refs` | leave and absence refs |
| `overtime_evidence_refs` | overtime approval/worked refs |
| `holiday_evidence_refs` | holiday calendar/pay treatment refs |
| `commission_evidence_refs` | commission event refs |
| `bonus_evidence_refs` | bonus approval refs |
| `deduction_evidence_refs` | deduction refs |
| `advance_repayment_refs` | advance recovery refs |
| `bank_account_ref` | current masked/controlled payment account ref |
| `tax_contribution_rule_refs` | approved tax/super/CPF/pension refs |
| `benefits_refs` | benefit election/rule refs |
| `exception_refs` | blocking or review items |
| `evidence_trust_summary` | trust posture summary |
| `ready_for_payroll` | boolean readiness |
| `blocking_reasons` | bounded reason codes |
| `audit_refs` | audit/provenance refs |

## 5. Evidence Trust Model

Evidence trust statuses:

- verified,
- unverified,
- employee_claimed,
- manager_approved,
- system_detected,
- provider_detected,
- disputed,
- overridden,
- rejected,
- expired,
- stale,
- missing,
- requires_review,
- corrected,
- locked.

Payroll interpretation:

- verified evidence can feed payroll.
- manager_approved evidence can feed payroll if policy accepts manager approval.
- employee_claimed evidence can create draft or exception depending on policy.
- disputed evidence creates exception.
- missing evidence blocks payroll or permits estimate only where policy allows.
- overridden evidence requires authority and audit.
- stale evidence requires revalidation.
- expired evidence cannot be used without renewal or override.
- locked evidence cannot be changed without correction/reversal flow.

## 6. State Machines

### EmploymentStatus

Candidate
-> Offered
-> Accepted
-> Onboarding
-> Active
-> Probation
-> OnLeave
-> Suspended
-> Resigned
-> Terminated
-> Retired
-> Offboarded
-> Rehired

### PayrunStatus

SetupDraft
-> EvidenceCollecting
-> EvidenceReady
-> PayrollDraft
-> EmployeePreviewIssued
-> EmployeeAccepted
-> EmployeeDisputed
-> ExceptionReview
-> PendingApproval
-> Approved
-> PaymentInstructionCreated
-> SentToAccounting
-> SentToBank
-> Paid
-> Failed
-> Corrected
-> Reversed
-> Archived

### PayslipReviewStatus

DraftIssued
-> EmployeeAccepted
-> EmployeeDisputed
-> NoResponse
-> Resolved
-> Escalated
-> Finalized

### DisputeStatus

Opened
-> EvidenceReviewing
-> SystemCorrectionPrepared
-> EmployeeExplanationPrepared
-> SupervisorApprovalPending
-> PayrollApprovalPending
-> Resolved
-> Rejected
-> Escalated
-> CorrectedAfterPayment
-> Closed

### OvertimeApprovalStatus

Requested
-> AutoApproved
-> SupervisorPending
-> Approved
-> Denied
-> Expired
-> Worked
-> NotWorked
-> PayrollEligible
-> Disputed

### BankAccountChangeStatus

Requested
-> StepUpRequired
-> StepUpPassed
-> StepUpFailed
-> PendingCutoffReview
-> EffectiveCurrentPayrun
-> EffectiveNextPayrun
-> Rejected
-> Applied
-> Audited

### LeaveStatus

Requested
-> PendingApproval
-> Approved
-> Denied
-> Active
-> Completed
-> PaidOut
-> Cancelled

### ContractorBillingMode

Hourly
LumpSum
Milestone
Retainer
Project
Agency
Site

### ContractorPaymentStatus

Draft
-> EvidenceRequired
-> InvoiceReceived
-> HoursVerified
-> MilestoneVerified
-> ManagerApprovalPending
-> APApprovalPending
-> Approved
-> PaymentInstructionCreated
-> Paid
-> Disputed
-> Rejected

### PaymentInstructionStatus

Draft
-> LedgerValidationPending
-> FinanceApprovalPending
-> BankReady
-> SentToBank
-> Confirmed
-> PartiallyFailed
-> Failed
-> Reconciled
-> Cancelled

### EvidenceTrustStatus

Use the trust statuses from section 5.

## 7. Authority Matrix

| Action | Risk | Default Owner | Required Access | Step-Up Required | Approval Role | Simulation Required | Audit Required | PH1.WRITE Explanation |
|---|---|---|---|---|---|---|---|---|
| prepare payroll draft | high | Payroll | payroll.prepare | no by default | Payroll Officer/Manager | yes | yes | yes |
| approve payroll | critical | Payroll | payroll.approve | yes | Payroll Manager / Finance | yes | yes | yes |
| commit payroll | critical | Payroll | payroll.commit | yes | Payroll + Finance | yes | yes | yes |
| create payment instruction | critical | Payroll/AP | payment_instruction.create | yes | Payroll/AP + Finance | yes | yes | yes |
| execute bank transfer | critical | Finance/Bank Provider | payment.execute | yes | Finance/Bank authorized approver | yes | yes | yes |
| view salary | high private | Access/Payroll | salary.view | yes where policy requires | none or manager/HR | no for read; yes for export | yes | yes |
| edit salary | critical | Compensation/Payroll | salary.edit | yes | HR/Payroll/Finance | yes | yes | yes |
| change bank account | critical private | Payroll | bank_account.change | yes | employee + payroll policy | yes | yes | yes |
| change tax details | critical private | Payroll | tax_detail.change | yes | employee/payroll policy | yes | yes | yes |
| approve overtime | high | Compensation/Attendance | overtime.approve | maybe | Supervisor/Manager | yes | yes | yes |
| approve leave | high | HR/Leave | leave.approve | no/step-up by policy | Manager/HR | yes | yes | yes |
| approve final pay | critical | Payroll/HR | final_pay.approve | yes | HR + Payroll | yes | yes | yes |
| terminate employee | critical | HR | employment.terminate | yes | HR/authorized manager/legal where required | yes | yes | yes |
| accept resignation | high | HR | resignation.accept | yes where policy requires | HR/Manager | yes | yes | yes |
| approve contractor invoice | high | Contractor/AP | contractor_invoice.approve | yes by threshold | Manager/AP/Finance | yes | yes | yes |
| approve contractor overrun | high | Contractor/AP + Task/HWM | contractor_overrun.approve | maybe | Manager/Finance | yes | yes | yes |
| approve salary advance | high | Payroll/Finance | salary_advance.approve | yes | Payroll/Finance | yes | yes | yes |
| reschedule deduction | high | Payroll | deduction.reschedule | yes where policy requires | Payroll Manager | yes | yes | yes |
| reverse payroll | critical | Payroll | payroll.reverse | yes | Payroll + Finance | yes | yes | yes |
| correct payslip | high | Payroll | payslip.correct | maybe | Payroll | yes | yes | yes |
| send payslip | high private | Payroll + BCAST/DELIVERY | payslip.send | yes where policy requires | Payroll | yes | yes | yes |
| issue HR notice | critical | HR | hr_notice.issue | yes | HR/authorized manager | yes | yes | yes |
| change compensation rules | critical | Compensation | compensation_rule.change | yes | HR/Finance/Management | yes | yes | yes |
| override public holiday calendar | high compliance | Holiday/Compensation | holiday.override | yes | Payroll/HR/Compliance | yes | yes | yes |
| change location tracking policy | critical privacy | HR/Legal/Access | location_policy.change | yes | HR/Legal/Admin | yes | yes | yes |

## 8. Step-Up Verification Model

Allowed future step-up methods:

- face verification,
- fingerprint verification,
- secure passcode,
- approved device confirmation,
- manager/admin approval fallback,
- fallback for users without biometric phones,
- audit of step-up.

Biometrics must not be authority by themselves. Step-up proves stronger identity posture. Access/Authority still decides whether action is allowed.

Sensitive actions requiring step-up:

- bank account change,
- tax detail change,
- salary/private data view,
- payslip download,
- final pay approval,
- payment instruction approval,
- payroll commit,
- termination notice,
- contractor payment approval.

## 9. Privacy / Security Classification

| Field Class | Who May View | Who May Edit | Who May Approve | Audit | Retention | Employee Visibility |
|---|---|---|---|---|---|---|
| public | authorized tenant users | owner/admin | owner/admin | low | policy | visible where relevant |
| employee_private | employee, HR/Payroll where authorized | employee/HR/Payroll | HR/Payroll | yes | policy | visible to employee |
| manager_visible | manager/HR | manager/HR | manager/HR | yes | policy | summary where policy allows |
| HR_only | HR | HR | HR authority | yes | HR retention | limited by policy |
| Payroll_only | Payroll | Payroll | Payroll authority | yes | payroll retention | employee-safe summary |
| Finance_only | Finance | Finance | Finance authority | yes | finance retention | no unless policy |
| medical_or_sick_sensitive | HR/Leave limited | HR/Leave | HR/Leave | yes | strict | limited employee own-data |
| bank_sensitive | employee/Payroll limited | employee/Payroll | Payroll | yes | strict | masked own-data |
| tax_sensitive | employee/Payroll limited | employee/Payroll | Payroll | yes | strict | own-data |
| termination_sensitive | HR/legal/authorized manager | HR/legal | HR/legal | yes | legal retention | policy-controlled |
| candidate_sensitive | HR/recruitment | HR/recruitment | HR | yes | candidate retention | candidate rights by policy |
| contractor_commercial_sensitive | Contractor/AP/Finance | AP/Finance/contract owner | AP/Finance | yes | contract retention | contractor summary where policy |
| location_sensitive | Attendance/HR limited | Attendance/HR | HR/legal policy | yes | minimum necessary | employee own-data where policy |
| performance_sensitive | HR/HWM authorized | HR/HWM authorized | HR/manager | yes | HR retention | employee summary where policy |
| compensation_sensitive | HR/Comp/Payroll/Finance | Comp/Payroll | Comp/Finance/HR | yes | payroll/HR retention | employee-safe own-data |

## 10. Payroll Rule Engine Requirements

Future deterministic rule components:

- `PayrollRuleEngine`
- `JurisdictionRulePack`
- `CompanyPayrollPolicyPack`
- `EmployeePayProfile`
- `PayLineCalculationPacket`
- `PayrollExceptionPacket`
- `TaxContributionRulePacket`
- `BenefitRulePacket`
- `HolidayRulePacket`
- `OvertimeRulePacket`
- `CommissionRulePacket`
- `DeductionRulePacket`
- `FinalPayRulePacket`

GPT-5.5 may explain or propose. Deterministic `PayrollRuleEngine` calculates.

## 11. Jurisdiction / Tax / Super / CPF / Pension Model

Required future fields:

- country,
- state/region,
- employment type,
- tax residency,
- thresholds,
- employer contribution,
- employee contribution,
- effective dates,
- source references,
- manual override authority,
- rule versioning,
- annual refresh,
- management confirmation,
- mismatch override.

Rules must be source-backed or owner-approved. Annual refresh and effective-date handling are mandatory. Mismatches create exceptions, not guesses.

## 12. Location / Attendance Evidence Model

Supported future evidence methods:

- GPS,
- Wi-Fi presence,
- NFC,
- QR check-in,
- badge scan,
- device trust,
- kiosk,
- supervisor confirmation,
- site photo where lawful,
- manual exception.

Required controls:

- location spoofing handling,
- after-hours tracking prohibition,
- consent/policy/notice,
- retention limit,
- minimum necessary tracking,
- manual review,
- role-based access,
- audit.

Location evidence is attendance evidence, not a general surveillance license.

## 13. Payment Execution Boundary

Future boundary:

- Payroll creates employee pay instruction.
- AP creates contractor/vendor pay instruction.
- Accounting validates ledger.
- Finance approves payment where threshold requires.
- Bank/Payment Provider sends transfer.
- Payment Provider returns confirmation or failure.
- Accounting reconciles.
- Payroll/AP records outcome.
- PH1.BCAST/DELIVERY notifies payee where permitted.

Failure handling:

- bank failure creates `PaymentInstructionStatus.Failed`;
- partial payment creates `PartiallyFailed` and reconciliation case;
- wrong bank account detected creates hold, escalation, and correction path;
- accounting rejection blocks bank send;
- provider proof is required before marking paid.

Hard rule:

Accounting/Finance/Bank provider executes payment rail. Payroll creates employee pay instruction. AP creates contractor/vendor pay instruction. Payment execution requires authority, simulation, audit, and provider proof.

## 14. Failure Modes

| Failure Mode | Required Behavior |
|---|---|
| GPS missing | Use alternate attendance evidence or create exception. |
| clock-in missing | Ask employee/supervisor for correction evidence; do not auto-pay unverified hours unless policy allows. |
| employee forgot to clock out | Correction flow with evidence, supervisor approval if required. |
| bank validation fails | Block payment instruction; request correction. |
| tax rule missing | Block or escalate payroll; do not invent rule. |
| holiday calendar mismatch | Create holiday exception and require owner confirmation. |
| manager does not respond | Reminder/escalation path; payroll exception remains. |
| employee disputes before payment | Open dispute; pause affected line where policy requires. |
| employee disputes after payment | Open corrected-after-payment workflow. |
| payment provider fails | Payment failure workflow and employee-safe status. |
| partial bank failure | Reconcile, identify affected payees, alert Finance/Payroll. |
| accounting rejects payment file | Block bank send and route correction. |
| contractor invoice mismatches hours | Contractor/AP exception; do not pay automatically. |
| overtime approval missing | Exception; do not auto-pay as approved overtime. |
| final pay cannot be calculated | Escalate missing evidence; observe legal deadline. |
| handover incomplete | Apply policy/jurisdiction guardrail; do not unlawfully hold statutory pay. |
| employee refuses handover | Escalate HR; preserve lawful payment path. |
| legal final-pay deadline conflicts with handover requirement | Prioritize legal/payment owner decision and escalate handover separately. |
| payslip sent to wrong recipient risk | Block/send failure, incident audit, notify owner. |
| employee changes bank account after cutoff | Apply next payrun unless urgent override approved. |
| payroll already locked | Correction/reversal only; no silent edit. |
| wrong public holiday calendar | Recalculate affected payroll after owner-approved calendar correction. |
| commission source missing | Commission exception; Sales owner must provide evidence. |
| sales refund clawback | Apply approved commission clawback rule only. |
| tax reporting integration unavailable | Block/reporting exception and owner escalation. |

## 15. Audit Event Model

Required audit fields:

- `audit_event_id`
- `actor_id`
- `subject_id`
- `action`
- `old_value_ref`
- `new_value_ref`
- `source_evidence_refs`
- `authority_result`
- `simulation_id`
- `confirmation_ref`
- `timestamp`
- `reason_code`
- `approval_refs`
- `payment_refs`
- `employee_visible_summary`
- `internal_summary`
- `provider_refs`
- `device_refs`
- `step_up_refs`
- `tenant_id`
- `company_id`
- `jurisdiction`
- `retention_policy_ref`

Audit must preserve source evidence and distinguish employee-visible summaries from internal sensitive evidence.

## 16. Simulation Inventory

Required logical simulations:

```text
PAYROLL.EVIDENCE_BUNDLE_CREATE
PAYROLL.DRAFT_CREATE
PAYROLL.EXCEPTION_REVIEW
PAYROLL.PAYSLIP_PREVIEW_ISSUE
PAYROLL.PAYSLIP_DISPUTE_OPEN
PAYROLL.PAYSLIP_DISPUTE_RESOLVE
PAYROLL.APPROVE
PAYROLL.COMMIT
PAYROLL.CORRECTION_PREPARE
PAYROLL.CORRECTION_APPLY
PAYROLL.REVERSE
PAYROLL.FINAL_PAY_CALCULATE
PAYROLL.PAYMENT_INSTRUCTION_CREATE

HR.RECRUITMENT_AD_PREPARE
HR.CANDIDATE_CONTACT_PREPARE
HR.OFFER_PREPARE
HR.PROBATION_REVIEW_PREPARE
HR.PROBATION_CONFIRM
HR.PROBATION_NON_CONFIRM_REVIEW
HR.RESIGNATION_RECORD
HR.TERMINATION_PREPARE
HR.TERMINATION_EXECUTE
HR.OFFBOARDING_PREPARE
HR.REHIRE_REACTIVATE

EMPLOYEE.BANK_ACCOUNT_CHANGE
EMPLOYEE.ADDRESS_CHANGE
EMPLOYEE.PHONE_CHANGE
EMPLOYEE.TAX_DETAIL_CHANGE

OVERTIME.REQUEST
OVERTIME.APPROVE
OVERTIME.DENY
OVERTIME.EXTEND
OVERTIME.PAYROLL_APPLY

LEAVE.REQUEST
LEAVE.APPROVE
LEAVE.DENY
LEAVE.PAYOUT_CALCULATE

CONTRACTOR.EVIDENCE_BUNDLE_CREATE
CONTRACTOR.OVERRUN_ALERT
CONTRACTOR.EXTRA_HOURS_APPROVE
CONTRACTOR.MILESTONE_VERIFY
CONTRACTOR.INVOICE_APPROVE
CONTRACTOR.PAYMENT_INSTRUCTION_CREATE
CONTRACTOR.ACCESS_EXPIRE

COMPENSATION.RATE_PROPOSE
COMPENSATION.RATE_UPDATE
COMPENSATION.OVERRIDE_REQUEST
COMPENSATION.OVERRIDE_APPROVE
COMPENSATION.COMMISSION_RULE_VALIDATE
COMPENSATION.BONUS_REVIEW

HOLIDAY.CALENDAR_PROPOSE
HOLIDAY.CALENDAR_CONFIRM
HOLIDAY.CALENDAR_OVERRIDE

PAYMENT.LEDGER_VALIDATE
PAYMENT.BANK_SEND
PAYMENT.CONFIRMATION_RECORD
PAYMENT.RECONCILE
PAYMENT.FAILURE_REVIEW
```

## 17. Provider / Mock Boundary

Future fake providers for tests:

- fake bank provider,
- fake accounting provider,
- fake GPS/location provider,
- fake roster provider,
- fake attendance provider,
- fake task provider,
- fake tax rule provider,
- fake holiday provider,
- fake recruitment source,
- fake payslip delivery provider,
- fake contractor invoice provider.

All providers must have:

- provider-off proof,
- fake-provider deterministic proof,
- failure-mode proof,
- audit proof.

No provider may become canonical payroll/HR truth.

## 18. Staged Build Order

Required stages:

1. Stage 1: owner map + preservation ledger
2. Stage 2: evidence bundle contracts
3. Stage 3: evidence trust model
4. Stage 4: payroll exception state machine
5. Stage 5: overtime approval state machine
6. Stage 6: bank account change + step-up flow
7. Stage 7: payslip preview/dispute flow
8. Stage 8: final pay/offboarding packet flow
9. Stage 9: contractor/AP boundary packet flow
10. Stage 10: accounting/payment handoff stub
11. Stage 11: jurisdiction/tax/holiday rule packs
12. Stage 12: privacy/security classification
13. Stage 13: audit model
14. Stage 14: full integration tests
15. Stage 15: JD live acceptance pack

## 19. Acceptance Test Matrix

| Test | Expected Proof |
|---|---|
| employee works normal shift -> payroll line created | verified roster + attendance evidence creates payroll draft line |
| employee works approved overtime -> paid correct multiplier | approved overtime and rule pack produce correct pay line |
| employee works unapproved overtime -> exception, not paid automatically | exception created; no automatic overtime pay |
| employee missing clock-in -> exception | missing evidence blocks or escalates by policy |
| employee forgot clock-out -> correction flow | correction state and approval captured |
| employee changes bank account -> step-up required | step-up evidence and audit required |
| bank account changed after cutoff -> next payrun unless urgent override | cutoff policy applied |
| draft payslip issued -> employee accepts -> payroll proceeds | accepted status gates next step |
| draft payslip disputed -> dispute case created | dispute state and evidence refs created |
| employee dispute resolved in employee favor -> draft corrected | corrected draft and audit |
| employee dispute rejected with evidence -> explanation given | PH1.WRITE employee-safe explanation |
| unresolved dispute -> escalated to manager/payroll | escalation refs |
| salary advance deduction applies | deduction evidence included |
| employee requests deduction delay -> approval flow | policy/approval path |
| contractor reaches 90% hours -> overrun alert | contractor overrun alert |
| contractor milestone verified -> AP payment ready | milestone evidence and AP instruction |
| terminated employee future shifts removed | roster removal handoff |
| resigned employee open tasks reallocated | Task/HWM handoff |
| final pay blocked by missing evidence where legally allowed | block reason and legal guardrail |
| final pay cannot be blocked where statutory deadline requires payment | lawful path overrides handover block |
| public holiday calendar confirmed -> holiday pay applied | holiday rule used |
| holiday calendar override -> affected payroll recalculated | recalculation and audit |
| tax rule missing -> payroll blocked/escalated | compliance exception |
| payment provider fails -> payment failure workflow | failure state and alert |
| partial bank payment fails -> reconciliation and alert | partial failure handling |
| payslip secure delivery proof | delivery proof and access check |
| employee cannot see another employee's salary | Access denial |
| unauthorized manager cannot approve payroll | authority denial |
| GPT-5.5 proposal cannot calculate final payroll | provider output rejected as final truth |
| PH1.WRITE produces employee-safe payroll explanation | output from validated evidence only |
| no old invalid names introduced | naming scan proof |

## 20. Preservation Ledger

| Source Concept / Function / Gap | Source A/B | Preserved In Document | Treatment | Notes |
|---|---|---|---|---|
| Payroll/HR automation umbrella | Source A Block 1 | Payroll + HR Automation Umbrella | preserved and expanded | Evidence-driven umbrella retained. |
| Selene continuously manages payroll evidence | Source A Block 1 | Umbrella; Evidence Fabric | expanded | Formalized as Evidence Fabric. |
| Reduce human payroll preparation | Source A Block 1 | Umbrella | preserved | Exception-first principle added. |
| Gather roster/attendance/task/leave/comp/tax/bank/accounting evidence | Source A Block 1 | Umbrella; Evidence Fabric | expanded | Mapped to packets. |
| Resolve routine issues directly | Source A Block 1 | Umbrella; Payrun/Dispute | preserved | Direct issue resolution retained. |
| Escalate exceptions | Source A Block 1 | Umbrella; Evidence Fabric | expanded | Exception and failure models added. |
| Send approved payment instructions to Accounting/Banking | Source A Block 1 | Umbrella; Payrun/Dispute; Evidence Fabric | expanded | Payment boundary hardened. |
| Bank account setup/change | Source A Block 1 | Umbrella; Evidence Fabric | expanded | Step-up and cutoff added. |
| Employee self-service | Source A Block 1 | Umbrella; Payrun/Dispute | preserved | Risk classes added. |
| Payment execution summary | Source A Block 1 | Umbrella; Evidence Fabric | expanded | Finance/Accounting/Bank owners explicit. |
| No orchestration truth owner | Source B 2 | Umbrella; Evidence Fabric | converted into law | Selene coordinates only. |
| HR lifecycle automation | Source A Block 2 | HR Recruitment/Offboarding | preserved | Full lifecycle listed. |
| Recruitment job ads | Source A Block 2 | HR Recruitment/Offboarding | preserved | Region/location support retained. |
| Job-site/talent-source search | Source A Block 2; Source B 24 | HR Recruitment/Offboarding | expanded | Lawful/policy-approved only. |
| Candidate outreach | Source A Block 2; Source B 24 | HR Recruitment/Offboarding | expanded | Approved channels, consent, anti-spam. |
| Resume/phone/email collection | Source A Block 2 | HR Recruitment/Offboarding | preserved | Candidate conversation included. |
| Candidate screening | Source A Block 2; Source B 25 | HR Recruitment/Offboarding | expanded | Approved/forbidden criteria. |
| Bias/privacy controls | Source A Block 2; Source B 24-25 | HR Recruitment/Offboarding | expanded | Protected traits warning included. |
| Probation management | Source A Block 2; Source B 26 | HR Recruitment/Offboarding | expanded | Evidence and scorecard requirement. |
| Probation pass/extend/fail recommendation | Source A Block 2 | HR Recruitment/Offboarding | preserved | Protected gate included. |
| Resignation flow | Source A Block 2 | HR Recruitment/Offboarding | preserved | Handover/final pay/access handoff retained. |
| Manager termination flow | Source A Block 2; Source B 12 | HR Recruitment/Offboarding | expanded | Reason categories, approvals, notice boundary. |
| Respectful employee notification | Source A Block 2 | HR Recruitment/Offboarding | preserved | PH1.WRITE owner added. |
| Work reallocation on exit | Source A Block 2; Source B 27 | HR Recruitment/Offboarding | expanded | Candidate ranking criteria added. |
| Records retention history | Source A Block 2 | HR Recruitment/Offboarding | preserved | Sensitive retention controls added. |
| Payroll evidence inputs | Source A Block 3 | Payrun/Dispute; Evidence Fabric | expanded | Converted to bundle fields. |
| Payrun lifecycle | Source A Block 3 | Payrun/Dispute; Evidence Fabric | expanded | State machine added. |
| Payslip preview before payment | Source A Block 3; Source B 20 | Payrun/Dispute; Evidence Fabric | expanded | Review status, timing need, no-response policy. |
| Employee agrees/disputes | Source A Block 3 | Payrun/Dispute | preserved | Dispute states added. |
| Dispute types: hours/overtime/leave/commission/bonus/deduction/tax/bank/final pay | Source A Block 3 | Payrun/Dispute | preserved | Evidence checks listed. |
| Selene presents evidence | Source A Block 3 | Payrun/Dispute; Evidence Fabric | expanded | Trust model added. |
| Fix system mistakes | Source A Block 3 | Payrun/Dispute | preserved | Correction flow added. |
| Explain when employee mistaken | Source A Block 3 | Payrun/Dispute | preserved | PH1.WRITE employee-safe wording. |
| Unresolved disputes escalate | Source A Block 3 | Payrun/Dispute; Evidence Fabric | preserved | Escalation state. |
| Payment confirmation sent to employee | Source A Block 3 | Payrun/Dispute; Evidence Fabric | preserved | Payment confirmation packet. |
| Salary advances/deductions rescheduled only with policy/approval | Source A Block 3; Source B 19 | Payrun/Dispute; Leave/Benefits; Evidence Fabric | expanded | Minimum net pay, cutoff, tax/accounting. |
| Accounting/bank integration | Source A Block 3; Source B 8 | Payrun/Dispute; Evidence Fabric | expanded | Payment authority chain. |
| Payroll troubleshooting assistant | Source A Block 3 | Payrun/Dispute | preserved | PH1.D + PH1.WRITE boundary. |
| Management setup country/region/industry/company size | Source A Block 4 | Compensation/Earnings | preserved | Source-backed rule warning. |
| Hourly/overtime/weekend/holiday rates | Source A Block 4 | Compensation/Earnings | preserved | Pay rule owner split. |
| Allowances, commissions, bonuses, benefits | Source A Block 4; Source B 31-33 | Compensation/Earnings | expanded | Commission/Sales and bonus/benefit boundaries. |
| Selene researches/proposes market numbers | Source A Block 4 | Compensation/Earnings | preserved | Source-backed only. |
| Management confirms/changes | Source A Block 4 | Compensation/Earnings | preserved | Authority/audit added. |
| Updating rates over time | Source A Block 4 | Compensation/Earnings | expanded | Effective date and impact preview. |
| Overtime request/approval | Source A Block 4 | Compensation/Earnings; Evidence Fabric | expanded | Status machine and fatigue/rest gates. |
| Supervisor routing without burdening employee | Source A Block 4 | Compensation/Earnings | preserved | Simple status retained. |
| Approved overtime feeds payroll | Source A Block 4 | Compensation/Earnings; Evidence Fabric | preserved | Packetized. |
| Automatic clock-out/end-of-day control | Source A Block 4; Source B 18 | Compensation/Earnings | expanded | Policy gates and site-presence distinction. |
| Commission boundary with Sales | Source A Block 4; Source B 31 | Compensation/Earnings | expanded | Sales truth separated. |
| Holiday/special-day identification | Source A Block 4; Source B 30 | Compensation/Earnings | expanded | Observed/substitute/leave/weekend cases. |
| Tax/super/retirement/benefit rules | Source A Block 4; Source B 7 | Compensation/Earnings; Evidence Fabric | expanded | Jurisdiction model. |
| Leave policy setup | Source A Block 5; Source B 29 | Leave/Benefits/Final Pay | expanded | Accrual, carry-forward, cash-out, caps. |
| Leave evidence flow | Source A Block 5 | Leave/Benefits/Final Pay; Evidence Fabric | preserved | Packetized. |
| Public holiday handling | Source A Block 5 | Leave/Benefits/Final Pay; Compensation/Earnings | expanded | Owner split. |
| Final pay automation | Source A Block 5 | Leave/Benefits/Final Pay; Evidence Fabric | expanded | State and failure modes. |
| Handover before final pay | Source A Block 5; Source B 28 | Leave/Benefits/Final Pay | converted with guardrail | Lawful deadline warning added. |
| Employee advances/deductions | Source A Block 5 | Leave/Benefits/Final Pay; Payrun/Dispute | preserved | Approval and visibility. |
| Contractor types | Source A Block 5 | Leave/Benefits/Final Pay | preserved | All types retained. |
| Contractor tracking | Source A Block 5 | Leave/Benefits/Final Pay | preserved | Hours/milestones/progress/site/invoice/access. |
| Contractor payment hourly/milestone/lump-sum | Source A Block 5 | Leave/Benefits/Final Pay | preserved | AP boundary hardened. |
| Contractor access expiry | Source A Block 5 | Leave/Benefits/Final Pay | preserved | REM/BCAST handoff. |
| Direct bank/accounting integration | Source A Block 5 | Leave/Benefits/Final Pay; Evidence Fabric | expanded | Payment rail owner split. |
| Tax/reporting handoff | Source A Block 5 | Leave/Benefits/Final Pay | preserved | Source-backed compliance. |
| Payments include salary/final pay/contractor invoice/bonus/commission/reimbursement/advance/deductions/tax remittance | Source A Block 5 | Leave/Benefits/Final Pay | preserved | Payment categories retained. |
| No canonical engine map | Source B 1 | Evidence Fabric section 2 | converted into owner map | Future owners listed. |
| Dangerous orchestration line | Source B 2 | Evidence Fabric section 2; Umbrella section 3 | corrected | Orchestration coordinates only. |
| Evidence Fabric missing | Source B 3 | Evidence Fabric section 3 | created | Formal bridge added. |
| Evidence trust model missing | Source B 4 | Evidence Fabric section 5 | created | Trust statuses defined. |
| Location tracking unfinished | Source B 5 | Evidence Fabric section 12; Compensation section 7 | expanded | Methods/privacy/spoofing/retention. |
| Payroll calculation engine unspecified | Source B 6 | Evidence Fabric section 10 | converted | Deterministic rule components. |
| Tax/super/CPF/pension logic loose | Source B 7 | Evidence Fabric section 11 | expanded | Jurisdiction model. |
| Payment execution boundary not hard enough | Source B 8 | Evidence Fabric section 13 | hardened | Create/validate/approve/send/confirm/reconcile. |
| Authority matrix missing | Source B 9 | Evidence Fabric section 7 | created | Action/risk/access/step-up/simulation/audit. |
| Simulations not listed | Source B 10 | Evidence Fabric section 16 | created | Simulation inventory added. |
| Employee dispute state machine missing | Source B 11 | Evidence Fabric section 6 | created | DisputeStatus added. |
| Termination logic too broad | Source B 12 | HR Recruitment/Offboarding; Evidence Fabric | expanded | Categories and legal gates. |
| Contractor payment separation weak | Source B 13 | Leave/Benefits/Final Pay; Evidence Fabric | hardened | Contractor/AP owner split. |
| Acceptance-test plan missing | Source B 14 | Evidence Fabric section 19 | created | Test matrix added. |
| Roster-to-payroll reconciliation missing | Source B 15 | Evidence Fabric sections 3-4; Payrun | converted | Roster/attendance refs. |
| Break tracking missing | Source B 16 | Compensation section 6; Evidence Fabric | preserved | Break logic listed. |
| Fatigue/rest rules missing | Source B 17 | Compensation section 6 | added | Limits and safety. |
| Auto clock-out gates too loose | Source B 18 | Compensation section 7 | expanded | Policy/grace/manual/site distinction. |
| Salary advance incomplete | Source B 19 | Payrun; Leave/Benefits | expanded | Eligibility/approval/min net pay. |
| Payslip preview timing undefined | Source B 20 | Payrun; Evidence Fabric | expanded | No-response/cutoff state noted. |
| Secure payslip delivery undefined | Source B 21 | Payrun section 10 | created | Channel/identity/expiry/wrong-recipient. |
| Employee self-service too broad | Source B 22 | Umbrella section 6 | classified | Risk classes. |
| Bank cutoff rules missing | Source B 23 | Umbrella; Evidence Fabric state machine | added | Effective current/next payrun. |
| Recruitment compliance gaps | Source B 24 | HR Recruitment section 2 | expanded | Consent/anti-spam/platform/privacy. |
| Candidate screening unfinished | Source B 25 | HR Recruitment section 3 | expanded | Criteria/forbidden/audit. |
| Probation scorecard missing | Source B 26 | HR Recruitment section 4; Evidence Fabric preservation | added as requirement | Scorecard/policy needed. |
| Work reallocation unfinished | Source B 27 | HR Recruitment section 7 | expanded | Capacity/skill/certification/availability. |
| Handover-final-pay guardrails missing | Source B 28 | Leave/Benefits section 6 | corrected | Legal deadline guardrail. |
| Leave policy generic | Source B 29 | Leave/Benefits section 2 | expanded | Policy fields. |
| Public holiday logic unfinished | Source B 30 | Compensation section 9 | expanded | Observed/substitute/weekend/leave. |
| Commission integration unfinished | Source B 31 | Compensation section 8 | expanded | Sales source, clawback, split, periods. |
| Bonus logic unfinished | Source B 32 | Compensation section 11 | expanded | Bonus eligibility/payment/tax. |
| Benefits logic unfinished | Source B 33 | Compensation section 11; Leave/Benefits | expanded | Eligibility/contribution/provider. |
| Multi-company/multi-region missing | Source B 34 | Evidence Fabric bundle fields and jurisdiction model | added | tenant/company/jurisdiction/payroll group. |
| No staged build order | Source B 35 | Evidence Fabric section 18 | created | 15 stages. |
| No file-scope approval | Source B 36 | Evidence Fabric section 0 and future build law | converted | This docs task stays approved-scope only; future implementation needs scope. |
| No packet schemas | Source B 37 | Evidence Fabric sections 3-4 | created | Bundle and packet inventory. |
| No enum/state definitions | Source B 38 | Evidence Fabric section 6 | created | Required state machines. |
| No failure modes | Source B 39 | Evidence Fabric section 14 | created | Failure table. |
| No mock/provider boundary | Source B 40 | Evidence Fabric section 17 | created | Fake providers and proof. |
| No audit event model | Source B 41 | Evidence Fabric section 15 | created | Audit fields. |
| No privacy/security classification | Source B 42 | Evidence Fabric section 9 | created | Field classes table. |
| No live smoke/acceptance proof plan | Source B 43 | Evidence Fabric section 19 | created | Acceptance test matrix. |
| PayrollEvidenceBundle is biggest missing piece | Source B conclusion | Evidence Fabric sections 3-4 | created | Central bridge. |
| Do not hand whole canvas to Codex as build prompt | Source B final answer | Evidence Fabric scope and staged build order | preserved | Future slices only. |

No source function or readiness issue is intentionally dropped.

## 21. What Must Not Happen

- no five-block function dropped,
- no 43-point revision issue dropped,
- no Payroll mega-engine swallowing HR,
- no HR engine calculating payroll,
- no Compensation engine executing payroll,
- no Finance/Accounting engine owning employee payroll truth,
- no Scheduler/Roster/Attendance calculating pay,
- no Task/HWM calculating pay,
- no Access role string exposing payroll/private HR fields,
- no GPT-5.5 final payroll calculation,
- no payment execution without payment-owner proof,
- no bank account change without step-up,
- no employee data history erased,
- no contractor treated as employee payroll by default,
- no implementation from this document alone.
