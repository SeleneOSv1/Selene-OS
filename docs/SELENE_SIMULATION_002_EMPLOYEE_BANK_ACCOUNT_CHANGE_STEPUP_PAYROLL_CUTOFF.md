# SIMULATION 002 — Employee Bank Account Change + Step-Up Verification + Payroll Cutoff Handling

DOCUMENT STATUS:
SIMULATION_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION
PENDING_REPO_TEST_MAPPING

```text id="sim002_header"
Simulation ID:
PAYROLL.EMPLOYEE_BANK_ACCOUNT_CHANGE_STEPUP_CUTOFF.V1

Document status:
SIMULATION_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION
PENDING_REPO_TEST_MAPPING

Purpose:
Show the real end-to-end process for Tom changing the bank account where his salary is paid, including private payroll classification, identity/session checks, Access/Governance scope validation, Face ID or secure passcode step-up, country-specific bank field collection, confirmation before save, payroll cutoff handling, old bank account history retention, Payroll owner update, audit, optional notification, failure branches, and future test assertions.

Primary user-facing goal:
Tom should feel like he is doing a sensitive but normal employee self-service task with a calm assistant, not trying to sneak through a finance vault with a sticky note and hope.

Main owners:
PH1.D / GPT-5.5 = understands messy human request and proposes meaning; drafts natural wording
PH1.N = extracts fields and ambiguity candidates
PH1.X = validates route/risk/lane and protected/private posture
PH1.WRITE = final user-facing wording
Access/Governance = identity scope, self-service permission, private payroll access, authority escalation if someone tries to change another employee's account
Payroll = payroll profile, bank account change request, cutoff decision, effective payrun, bank account history
Device/Human Presence = Face ID, fingerprint, secure passcode, approved device confirmation, step-up evidence
Audit = every sensitive/private step and no raw bank details in logs
PH1.REM = reminder/follow-up timing if a pending correction, approval, or payroll setup issue needs follow-up
PH1.BCAST / PH1.DELIVERY = optional payroll/admin notification delivery where company policy requires it
```

---

## 0. Simulation Header

```text id="sim002_scope"
Simulation type:
Acceptance simulation design.

Build class:
Docs-only / cross-engine process simulation.

What this simulation proves:
- Selene can understand an employee's bank account change request in human language.
- PH1.D and PH1.N can propose and extract meaning without owning the action.
- PH1.X can classify the request as private payroll self-service.
- Access/Governance can confirm the employee is changing only their own payroll profile.
- Device/Human Presence can require step-up before any bank-sensitive data is saved.
- Payroll owns the bank account change and payrun cutoff effect.
- Old bank account history is retained.
- Audit records the change without raw bank details.
- PH1.WRITE explains the outcome in simple English.
- BCAST/DELIVERY is used only for optional notifications when policy requires.

What this simulation does not implement:
- no runtime code,
- no Payroll engine,
- no HR engine,
- no bank provider,
- no migration,
- no Rust packet/schema,
- no client UI,
- no adapter route,
- no tests yet.

Repo-truth caution:
The Payroll/HR extraction says current repo truth does not prove a complete Payroll runtime engine. The packets and database records below are future logical simulation shapes pending repo mapping.
```

---

## 1. Starting Database Assumptions

These records already exist before Tom asks anything.

### `tenant_companies`

```text id="tenant_companies_sim002"
tenant_id: tenant_abc
company_id: company_abc_wines
company_name: ABC Wines
country: Australia
region: NSW
timezone: Australia/Sydney
status: active
company_policy_pack_ref: policy_pack_abc_v7
```

### `employee_profiles`

```text id="employee_profiles_sim002"
employee_id: emp_tom_001
user_id: user_tom
legal_name: Thomas Richards
employment_status: active
company_id: company_abc_wines
position_id: pos_warehouse_supervisor
payroll_profile_id: payroll_profile_tom_001
primary_device_ref: device_tom_iphone_001
```

### `payroll_profiles`

```text id="payroll_profiles_sim002"
payroll_profile_id: payroll_profile_tom_001
employee_id: emp_tom_001
pay_frequency: weekly
payroll_group: payroll_group_warehouse_weekly
current_bank_account_ref: bank_ref_tom_old_001
current_bank_account_last4: 5678
bank_account_region_schema: AU_BANK_DETAILS
payroll_status: active
current_payrun_lock_status: locked
pending_bank_account_change_ref: null
```

### `payrun_calendar`

```text id="payrun_calendar_sim002"
payrun_id: payrun_2026_06_07
pay_period_start: 2026-06-01
pay_period_end: 2026-06-07
payroll_cutoff_at: 2026-06-06T17:00:00+10:00
pay_date: 2026-06-10
status: locked

next_payrun_id: payrun_2026_06_14
next_pay_period_start: 2026-06-08
next_pay_period_end: 2026-06-14
next_payroll_cutoff_at: 2026-06-13T17:00:00+10:00
next_pay_date: 2026-06-17
next_status: evidence_collecting
```

Payrun statuses may include:

```text id="payrun_statuses_sim002"
open
evidence_collecting
locked
approved
paid
```

### `access_instances`

```text id="access_instances_sim002"
access_instance_id: access_tom_001
user_id: user_tom
company_id: company_abc_wines
permissions:
  - payroll.self.view
  - payroll.self.bank_account.update
  - employee.self.profile.view
can_update_own_bank_account: true
can_view_own_payroll_profile: true
can_update_other_employee_bank_account: false
status: active
```

### `device_profile`

```text id="device_profile_sim002"
device_ref: device_tom_iphone_001
user_id: user_tom
biometric_available: true
biometric_type: Face ID
secure_passcode_available: true
approved_device_confirmation_available: true
device_trust_status: trusted_primary_device
last_seen_at: 2026-06-07T10:16:00+10:00
```

### `bank_account_history`

```text id="bank_account_history_sim002"
employee_id: emp_tom_001
payroll_profile_id: payroll_profile_tom_001
current_ref: bank_ref_tom_old_001
current_last4: 5678
old_refs_retained:
  - bank_ref_tom_previous_000
audit_refs:
  - audit_bank_initial_setup_001
raw_bank_details_exposed_in_history: false
```

### `company_payroll_self_service_policy`

```text id="company_payroll_self_service_policy_sim002"
policy_id: payroll_self_service_abc_v3
company_id: company_abc_wines
employee_can_change_own_bank_account: true
bank_account_change_step_up_required: true
bank_account_change_confirmation_required: true
bank_account_change_before_cutoff_applies_current_payrun: true
bank_account_change_after_cutoff_applies_next_payrun: true
urgent_after_cutoff_override_allowed: true
urgent_after_cutoff_override_requires_payroll_finance_approval: true
notify_payroll_admin_on_bank_change: true
notify_manager_on_bank_change: false
retain_old_bank_account_history: true
audit_raw_bank_details_allowed: false
```

---

## 2. User Starts The Request

## Tom said

```text id="tom_starts_bank_change_sim002"
Selene, change the bank account my salary goes into.
```

## Internal probabilistic process

### PH1.D / GPT-5.5 proposal

This is **probabilistic**.

```text id="ph1d_proposal_sim002"
intent: employee_bank_account_change
likely_action: update salary payment bank account
subject_person_candidate: current authenticated speaker
target_owner_candidate: Payroll
risk_hint: private payroll data + bank-sensitive field + payment destination change
fields_likely_needed:
  - bank_name
  - country_specific_routing_number
  - account_number
  - account_name
  - confirmation
step_up_likely_required: true
confidence: high
```

GPT-5.5 does **not** update the bank account.

GPT-5.5 does **not** decide Tom is authorized.

GPT-5.5 does **not** decide payroll cutoff.

It proposes meaning and friendly wording only. Clever, useful, still not allowed near the money switch.

---

## 3. PH1.N Extraction

```text id="ph1n_extraction_sim002"
PayrollSelfServiceIntentCandidatePacket:
  intent_id: PAYROLL.EMPLOYEE_BANK_ACCOUNT_CHANGE
  person_candidate: Tom / current speaker
  user_candidate: user_tom
  action: change_salary_payment_bank_account
  target: own_payroll_profile
  company_candidate: company_abc_wines
  missing_fields:
    - bank_name
    - BSB
    - account_number
    - account_name
  country_specific_schema_candidate: AU_BANK_DETAILS
  ambiguity_flags:
    target_employee_ambiguous: false
    own_or_other_employee_unclear: false
    bank_country_unclear: false
    bank_details_missing: true
  confidence_bucket: HIGH
```

PH1.N may extract future bank fields after Tom provides them.

PH1.N must not save bank fields.

PH1.N must not treat a good-looking BSB as final bank truth.

---

## 4. PH1.X Route / Risk Validation

This is deterministic.

```text id="ph1x_validation_sim002"
request_lane: protected_private_self_service
data_scope: company_private + employee_private + bank_sensitive
action_effect: changes future salary payment destination
private_payroll_data: true
protected_sensitive_self_service: true
requires_authenticated_session: true
requires_access_check: true
requires_step_up: true
requires_payroll_owner: true
requires_confirmation_before_save: true
requires_payrun_cutoff_check: true
requires_audit: true
allowed_to_continue_before_gates_pass: false
```

Selene may keep talking to Tom.

Selene may not save or apply the bank account change until the required gates pass.

---

## 5. Access / Identity Check

Access/Governance checks the identity and scope before sensitive collection.

```text id="access_identity_check_sim002"
actor_user_id: user_tom
session_user_id: user_tom
employee_id: emp_tom_001
target_employee_id: emp_tom_001
company_id: company_abc_wines
employee_profile_status: active
payroll_profile_exists: true
action: payroll.self.bank_account.update
requested_scope: own_payroll_profile
permission_check:
  can_view_own_payroll_profile: true
  can_update_own_bank_account: true
  can_update_other_employee_bank_account: false
policy_check:
  employee_can_change_own_bank_account: true
access_result: allowed_pending_step_up
```

If Tom is changing his own account and policy allows it, the flow continues.

If Tom tries to change someone else's account, Access blocks or routes to the Master Access authority escalation flow if company policy allows. Payroll does not guess the approver and does not continue without a valid authority result.

---

## 6. Step-Up Verification

Device/Human Presence determines which proof is available.

```text id="device_stepup_capability_sim002"
DeviceStepUpCapabilityPacket:
  user_id: user_tom
  device_ref: device_tom_iphone_001
  trusted_primary_device: true
  face_id_available: true
  fingerprint_available: false
  secure_passcode_available: true
  approved_device_confirmation_available: true
  preferred_step_up_method: face_id
```

## Selene said

```text id="selene_faceid_prompt_sim002"
Sure — because this changes where your salary is paid, I need a quick Face ID check first.
```

If no biometric is available:

```text id="selene_passcode_prompt_sim002"
Sure — this is sensitive, so I'll use your secure passcode instead.
```

### Future logical packet: `StepUpVerificationRequestPacket`

```text id="stepup_request_packet_sim002"
StepUpVerificationRequestPacket:
  step_up_request_id: stepup_req_bank_001
  user_id: user_tom
  employee_id: emp_tom_001
  device_ref: device_tom_iphone_001
  action: payroll.self.bank_account.update
  sensitive_field_class: bank_sensitive
  preferred_method: face_id
  fallback_methods:
    - secure_passcode
    - approved_device_confirmation
  expires_at: 2026-06-07T10:21:00+10:00
  audit_ref: audit_stepup_request_001
```

### Future logical packet: `StepUpVerificationResultPacket`

```text id="stepup_result_packet_sim002"
StepUpVerificationResultPacket:
  step_up_result_id: stepup_bank_001
  request_id: stepup_req_bank_001
  user_id: user_tom
  device_ref: device_tom_iphone_001
  method: face_id
  result: passed
  liveness_or_device_trust_result: trusted_device_passed
  completed_at: 2026-06-07T10:17:18+10:00
  audit_ref: audit_stepup_result_001
```

Important rule:
Step-up proof is evidence.
Step-up proof does not grant payroll authority by itself.
Payroll still owns the update.

---

## 7. Country-Specific Bank Field Collection

The company is in Australia, so Selene uses the Australia bank details schema.

For Australia, Selene needs:

```text id="au_bank_schema_sim002"
AU_BANK_DETAILS:
  bank_name: required
  BSB: required
  account_number: required
  account_name: required
  branch: optional unless company policy says otherwise
```

## Selene said

```text id="selene_au_bank_prompt_sim002"
Great. For Australia, I need the bank name, BSB, account number, and account name.
```

## Tom said

```text id="tom_bank_details_sim002"
Commonwealth Bank, BSB 062000, account 87654321, account name Thomas Richards.
```

### PH1.N extracts fields

```text id="ph1n_bank_extract_sim002"
BankAccountFieldCandidatePacket:
  bank_name_candidate: Commonwealth Bank
  bsb_candidate: "062000"
  account_number_candidate: "87654321"
  account_name_candidate: Thomas Richards
  branch_candidate: null
  country_schema: AU_BANK_DETAILS
  ambiguity_flags:
    bsb_ambiguous: false
    account_number_ambiguous: false
    account_name_missing: false
  confidence_bucket: HIGH
```

### Deterministic validator checks format

```text id="bank_field_validation_sim002"
BankAccountFieldValidationPacket:
  country_schema: AU_BANK_DETAILS
  bank_name_present: true
  bsb_format_valid: true
  account_number_format_valid: true
  account_name_present: true
  account_name_matches_employee_legal_name: true
  branch_required: false
  raw_bank_details_storage_required: encrypted_only
  bank_validation_status: format_valid_pending_payroll_owner_acceptance
```

Do not claim live bank validation unless repo/provider proof exists.

If a future bank provider exists, the status may become:

```text id="future_provider_bank_validation_sim002"
bank_validation_status: provider_validated
provider_ref: bank_provider_validation_001
```

For this simulation, the safe status is:

```text id="safe_bank_validation_status_sim002"
bank_validation_status: format_valid_pending_payroll_owner_acceptance
```

---

## 8. Confirmation Before Save

Selene must confirm before updating.

## Selene said

```text id="selene_bank_confirm_sim002"
I'll update your salary account to Commonwealth Bank, account ending 4321. Should this be used for your future salary payments?
```

## Tom said

```text id="tom_bank_confirm_yes_sim002"
Yes.
```

### Confirmation packet

```text id="bank_confirmation_packet_sim002"
BankAccountChangeConfirmationPacket:
  confirmation_id: confirm_bank_change_001
  employee_id: emp_tom_001
  payroll_profile_id: payroll_profile_tom_001
  safe_display_bank_name: Commonwealth Bank
  safe_display_last4: 4321
  confirmation_text_ref: ph1write_confirm_bank_001
  confirmed_by_user_id: user_tom
  confirmed_at: 2026-06-07T10:18:11+10:00
  audit_ref: audit_bank_confirm_001
```

No confirmation means no save.

---

## 9. Payrun Cutoff Check

Payroll checks whether the current payrun is still open.

```text id="payrun_cutoff_check_main_sim002"
PayrollCutoffCheckPacket:
  employee_id: emp_tom_001
  payroll_profile_id: payroll_profile_tom_001
  requested_at: 2026-06-07T10:18:20+10:00
  current_payrun_id: payrun_2026_06_07
  current_payrun_status: locked
  payroll_cutoff_at: 2026-06-06T17:00:00+10:00
  after_cutoff: true
  next_payrun_id: payrun_2026_06_14
  cutoff_result: current_payrun_locked_apply_next_payrun
  urgent_override_available: true
  urgent_override_requires:
    - Payroll approval
    - Finance approval if payment file is affected
    - simulation
    - audit
```

### Case A: current payrun open before cutoff

```text id="cutoff_case_a_sim002"
current_payrun_status: open
after_cutoff: false
policy_allows_current_payrun_change: true
effective_payrun_id: payrun_2026_06_07
effective_result: applies_current_payrun
```

Selene may say:

```text id="cutoff_open_wording_sim002"
This change is before payroll cutoff, so it will apply to the upcoming payrun.
```

### Case B: payrun locked or after cutoff

```text id="cutoff_case_b_sim002"
current_payrun_status: locked
after_cutoff: true
effective_payrun_id: payrun_2026_06_14
effective_result: applies_next_payrun
```

Main path wording:

```text id="cutoff_locked_wording_sim002"
Your next payroll is already locked, so I'll apply this new account from the following payrun. Your current pay will still use the account ending 5678.
```

### Case C: payrun already paid

```text id="cutoff_case_c_sim002"
current_payrun_status: paid
effective_payrun_id: payrun_2026_06_14
effective_result: future_payruns_only
```

### Case D: urgent exception requested

```text id="cutoff_case_d_sim002"
urgent_override_requested: true
required_approval:
  - payroll_admin
  - finance_owner_if_payment_instruction_exists
simulation_required: true
audit_required: true
status: approval_required_before_change_can_affect_locked_payrun
```

Urgent override is not part of the main path.

---

## 10. Payroll Owner Update

Payroll owns the update. Access, Device/Human Presence, PH1.D, and PH1.N only provide gates or evidence.

### Future logical record: `payroll_bank_account_change_request`

```text id="payroll_bank_change_request_sim002"
payroll_bank_account_change_request:
  change_request_id: bank_change_req_tom_001
  employee_id: emp_tom_001
  payroll_profile_id: payroll_profile_tom_001
  old_bank_account_ref: bank_ref_tom_old_001
  new_bank_account_ref: bank_ref_tom_new_001
  old_last4: 5678
  new_last4: 4321
  effective_payrun_id: payrun_2026_06_14
  effective_date: 2026-06-08
  cutoff_result: current_payrun_locked_apply_next_payrun
  step_up_ref: stepup_bank_001
  confirmation_ref: confirm_bank_change_001
  requested_by_user_id: user_tom
  status: pending_effective_next_payrun
  audit_ref: audit_bank_change_request_001
```

### Future logical update: `payroll_profiles`

```text id="payroll_profile_update_sim002"
payroll_profiles:
  payroll_profile_id: payroll_profile_tom_001
  current_bank_account_ref: bank_ref_tom_old_001
  current_bank_account_last4: 5678
  pending_bank_account_change_ref: bank_change_req_tom_001
  pending_bank_account_last4: 4321
  pending_bank_account_effective_payrun_id: payrun_2026_06_14
  payroll_status: active
```

Because the main path applies next payrun, the current bank account remains active until the effective payrun boundary.

If the change applies immediately before cutoff:

```text id="payroll_profile_immediate_update_sim002"
payroll_profiles:
  current_bank_account_ref: bank_ref_tom_new_001
  current_bank_account_last4: 4321
  pending_bank_account_change_ref: null
  updated_for_payrun_id: payrun_2026_06_07
```

### Future logical update: `bank_account_history`

```text id="bank_history_update_sim002"
bank_account_history:
  employee_id: emp_tom_001
  payroll_profile_id: payroll_profile_tom_001
  old_bank_retained_ref: bank_ref_tom_old_001
  old_bank_retained_last4: 5678
  new_bank_ref: bank_ref_tom_new_001
  new_bank_last4: 4321
  change_reason: employee_self_service_bank_account_change
  step_up_ref: stepup_bank_001
  audit_refs:
    - audit_bank_change_request_001
    - audit_bank_history_retained_001
  old_data_deleted: false
  raw_bank_details_in_audit: false
```

Rule:
New data becomes current or pending current.
Old data becomes historical.
Nothing sensitive is silently erased.

---

## 11. PH1.WRITE Final Human Confirmation

PH1.WRITE owns the final wording. It receives validated truth from Payroll and Access.

## Main path Selene said

```text id="final_confirmation_locked_sim002"
Done — I've saved the new account ending 4321. Because the current payrun is already locked, it will be used from the next payrun after this one. Your current pay will still go to the account ending 5678.
```

Optional second sentence:

```text id="final_confirmation_history_sim002"
I'll keep the previous account in payroll history, but it won't be used once the change takes effect.
```

If applied before cutoff:

```text id="final_confirmation_open_sim002"
Done — your salary will now be paid to the account ending 4321 from the upcoming payrun.
```

If the employee asks why it cannot apply now:

```text id="cutoff_explanation_sim002"
Payroll has already locked this payrun, so changing the payment account now would need a payroll override. I can help request that if it's urgent.
```

---

## 12. Audit Events

Audit must record the sensitive path without exposing raw bank details.

```text id="audit_events_sim002"
AuditEvent:
  audit_event_id: audit_bank_change_request_001
  actor_user_id: user_tom
  subject_employee_id: emp_tom_001
  action: payroll_bank_account_change_requested
  old_value_ref: bank_ref_tom_old_001
  new_value_ref: bank_ref_tom_new_001
  old_value_safe_display: account ending 5678
  new_value_safe_display: account ending 4321
  step_up_result_ref: stepup_bank_001
  cutoff_result: current_payrun_locked_apply_next_payrun
  effective_payrun_id: payrun_2026_06_14
  timestamp: 2026-06-07T10:18:25+10:00
  device_ref: device_tom_iphone_001
  reason_code: employee_self_service_bank_account_change
  ph1write_confirmation_ref: ph1write_bank_change_confirm_001
  payroll_owner_update_ref: bank_change_req_tom_001
  tenant_id: tenant_abc
  company_id: company_abc_wines
  raw_bank_details_logged: false
```

Additional audit events:

```text id="audit_additional_sim002"
AuditEvent:
  audit_event_id: audit_stepup_result_001
  action: step_up_verification_passed
  method: face_id
  raw_biometric_data_logged: false

AuditEvent:
  audit_event_id: audit_bank_history_retained_001
  action: previous_bank_account_retained_in_history
  old_ref: bank_ref_tom_old_001
  raw_bank_details_logged: false

AuditEvent:
  audit_event_id: audit_payroll_cutoff_decision_001
  action: payroll_cutoff_decision_recorded
  cutoff_result: current_payrun_locked_apply_next_payrun
```

No raw bank details in audit logs.

No biometric template in audit logs.

No private payroll details visible to unauthorized users.

---

## 13. Optional Notifications

If company policy requires payroll/admin notification, PH1.BCAST / PH1.DELIVERY sends it.

Payroll remains the truth owner.

PH1.WRITE owns the notification wording.

PH1.BCAST / PH1.DELIVERY owns delivery only.

```text id="optional_notification_policy_sim002"
notify_payroll_admin_on_bank_change: true
notify_manager_on_bank_change: false
```

### Internal notification

```text id="payroll_admin_notification_packet_sim002"
DeliveryRequestPacket:
  delivery_id: delivery_payroll_admin_bank_change_001
  message_type: payroll_profile_change_notice
  recipient_role: payroll_admin
  company_id: company_abc_wines
  message_body_ref: ph1write_payroll_admin_bank_change_notice_001
  sensitive_content_policy: masked_bank_details_only
  requires_delivery_receipt: true
```

### PH1.WRITE notification text

```text id="payroll_admin_notification_wording_sim002"
Tom Richards updated his salary payment account. The new account takes effect from payrun PAY-2026-06-14. Step-up verification passed.
```

Do not notify Tom's manager by default unless policy requires it.

Do not expose full bank details.

---

## 14. Failure Branches

## A. Step-up fails

```text id="failure_stepup_fails_sim002"
step_up_result: failed
bank_account_update_allowed: false
audit_required: true
```

Selene says:

```text id="failure_stepup_fails_wording_sim002"
I couldn't verify you, so I can't change the bank account yet. You can try again or use the approved fallback.
```

## B. Device has no Face ID/fingerprint

```text id="failure_no_biometric_sim002"
face_id_available: false
fingerprint_available: false
secure_passcode_available: true
fallback_result: use_secure_passcode
```

Selene says:

```text id="failure_no_biometric_wording_sim002"
This is sensitive, so I'll use your secure passcode instead.
```

## C. Passcode fails

```text id="failure_passcode_fails_sim002"
secure_passcode_result: failed
retry_policy_remaining_attempts: 1
bank_account_update_allowed: false
audit_failed_attempt: true
```

Selene blocks the update and offers retry only under company policy.

## D. Bank details invalid format

```text id="failure_invalid_bsb_sim002"
bsb_candidate: "06200"
bsb_format_valid: false
bank_account_update_allowed: false
```

Selene says:

```text id="failure_invalid_bsb_wording_sim002"
That BSB doesn't look complete. It should be six digits. Can you check it and send it again?
```

## E. Name mismatch

```text id="failure_name_mismatch_sim002"
account_name_candidate: Tom R.
employee_legal_name: Thomas Richards
name_match_status: possible_mismatch
requires_confirmation_or_payroll_review: true
```

Selene says:

```text id="failure_name_mismatch_wording_sim002"
The account name doesn't exactly match your employee name. Is "Tom R." the correct account name, or should payroll review it before I save the change?
```

## F. Payrun locked

```text id="failure_payrun_locked_sim002"
current_payrun_status: locked
change_effect: next_payrun
urgent_override_possible: true
urgent_override_requires_payroll_finance_approval: true
simulation_required: true
```

Selene says:

```text id="failure_payrun_locked_wording_sim002"
Payroll has already locked this payrun, so this change will apply from the next payrun. If it's urgent, I can help request a payroll override.
```

## G. Employee attempts to change another employee's bank account

```text id="failure_other_employee_sim002"
actor_user_id: user_tom
target_employee_id: emp_sarah_001
permission_check: denied
escalation_policy_allowed: depends_on_company_policy
```

If escalation is allowed, Access/Governance uses the authority failure escalation law.

If escalation is forbidden, Selene says:

```text id="failure_other_employee_wording_sim002"
You don't currently have permission to change another employee's payroll details.
```

## H. Payroll profile missing

```text id="failure_payroll_profile_missing_sim002"
employee_id: emp_tom_001
payroll_profile_exists: false
bank_account_update_allowed: false
issue_created: payroll_setup_issue_tom_001
```

Selene says:

```text id="failure_payroll_profile_missing_wording_sim002"
Your payroll setup is not complete yet, so I can't update the payment account directly. I've created a payroll setup issue so this can be fixed.
```

## I. Tax/payroll owner unavailable

```text id="failure_payroll_owner_unavailable_sim002"
payroll_owner_available: false
status: safe_pending
bank_update_committed: false
reminder_or_follow_up_allowed: true
```

Selene says:

```text id="failure_payroll_owner_unavailable_wording_sim002"
I have your request, but payroll is not available to accept the change yet. I haven't saved the new account. I'll follow up when payroll can process it.
```

PH1.REM may schedule a follow-up if policy allows.

## J. Delivery/notification fails

```text id="failure_notification_delivery_sim002"
payroll_update_status: pending_effective_next_payrun
notification_delivery_status: failed
truth_impact: none
delivery_failure_recorded: true
```

Delivery failure does not undo Payroll truth.

## K. Employee changes mind

```text id="failure_employee_changes_mind_sim002"
pending_change_ref: bank_change_req_tom_001
effective_status: not_yet_effective
cancel_allowed: true
requires_step_up_for_cancel: policy_dependent
```

Selene says:

```text id="failure_changes_mind_wording_sim002"
No problem. Because the change has not taken effect yet, I can cancel the pending bank account change after verification.
```

## L. Employee disputes later

```text id="failure_later_dispute_sim002"
employee_claim: my salary went to the wrong account
case_type: PayrollProfileChangeReviewCase
related_refs:
  - bank_change_req_tom_001
  - audit_bank_change_request_001
  - payrun_2026_06_14
status: opened
```

Selene creates a PayrollDisputeCase or PayrollProfileChangeReviewCase and attaches the safe evidence.

---

## 15. What Was Probabilistic?

```text id="probabilistic_sim002"
PH1.D / GPT-5.5:
- understood "change the bank account my salary goes into"
- proposed employee bank account change intent
- drafted friendly wording
- helped explain payroll cutoff simply
- helped explain failed verification simply
- helped summarize a later dispute if one is opened

PH1.N:
- extracted the requested action
- linked target to current speaker / own payroll profile candidate
- extracted bank name, BSB, account number, and account name candidates
- detected ambiguity candidates and missing fields
```

Probabilistic did **not**:

```text id="probabilistic_did_not_sim002"
- approve the change
- validate final bank truth
- update Payroll
- decide cutoff
- bypass step-up
- decide Access scope
- expose private data
- write raw bank details into audit
```

---

## 16. What Was Deterministic?

```text id="deterministic_sim002"
PH1.X:
- route/risk/lane validation
- private payroll sensitive classification

Access/Governance:
- authenticated user check
- own-payroll-profile scope check
- company scope check
- self-service permission check
- authority escalation if someone tries to update another employee's bank account

Device/Human Presence:
- Face ID / fingerprint / secure passcode capability detection
- step-up verification evidence
- approved device confirmation evidence where needed

Payroll:
- payroll profile lookup
- country-specific bank schema acceptance
- bank account change request ownership
- payrun cutoff check
- effective payrun decision
- current or pending bank account ref update
- old bank account history retention

Audit:
- step-up request/result audit
- cutoff decision audit
- bank account change audit
- no raw bank details in audit

PH1.WRITE:
- final wording validation
- safe bank masking
- cutoff explanation
- denial/failure wording

PH1.BCAST/DELIVERY:
- optional payroll/admin notification delivery if policy requires

PH1.REM:
- follow-up timing for pending payroll owner availability, unresolved payroll setup issue, or dispute follow-up if needed
```

---

## 17. Test Assertions

A real future implementation test should prove:

```text id="test_assertions_sim002"
1. GPT-5.5 proposal alone cannot change bank account.
2. PH1.X classifies bank account change as private payroll sensitive action.
3. Step-up required before bank details are saved.
4. Face ID path works where available.
5. Passcode fallback works where biometrics unavailable.
6. Failed step-up blocks change.
7. AU bank field schema requires bank name, BSB, account number, account name.
8. Invalid BSB format blocks save.
9. Confirmation required before save.
10. Old bank account is retained in history.
11. No raw bank details in audit.
12. Payrun cutoff determines effective payrun.
13. Locked payrun applies change to future payrun.
14. Urgent override requires approval/simulation.
15. Employee cannot change another employee's bank account without authority.
16. Payroll profile missing creates setup issue.
17. PH1.WRITE produces safe human confirmation.
18. Optional notification uses BCAST/DELIVERY only.
19. Access/Payroll/Device/Audit ownership boundaries are respected.
20. No runtime implementation from simulation doc.
```

---

## 18. Final Simulation Sentence

This simulation proves that Selene can let an employee change their salary payment account through a human conversation while PH1.D only proposes wording, PH1.X classifies risk, Access validates identity/scope, Device/Human Presence proves step-up, Payroll owns the bank-account update and cutoff effect, PH1.WRITE explains the result, and Audit records the change without exposing raw bank details.
