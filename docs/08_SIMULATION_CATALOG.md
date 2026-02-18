# Simulation Catalog (Authoritative Inventory)

This file is the authoritative inventory of Selene simulations.

Hard rule
- The Simulation Catalog laws (schema + execution gates) live in `docs/05_OS_CONSTITUTION.md` (SCS.*) and `docs/04_KERNEL_CONTRACTS.md` (KC.7).
- The *simulation list* (add/rename/deprecate) must be maintained here.
- Simulation execution bindings must reference explicit `engine_id + capability_id` pairs from ACTIVE Engine Capability Maps (KC.7A). Missing/invalid capability binding fails closed.

Maintenance rule (hard)
Whenever a simulation is added, renamed, deprecated, or activated:
- Update this file.
- Update any referencing Process Blueprints (PBS.*) to reference the correct `simulation_id` + version.
- Ensure Access/Authority requirements remain explicit (no silent authority).

## Simulation Catalog Schema Lock (Item 5)

Status: `LOCKED`

Locked schema rules:
- Every simulation record must define all required contract fields from `KC.7` (directly or through explicit domain defaults in this file).
- `required_roles`, `preconditions`, `reads_tables[]`, `writes_tables[]`, `idempotency_key_rule`, and `audit_events` must be explicit; no `TBD` is allowed.
- Record-level DB binding overrides may only reduce table scope from the domain profile, never widen it.
- Any simulation record violating these rules remains `DRAFT` and cannot be executed.

## Catalog Index (Summary)

| simulation_id | type | owning_domain | purpose | status | version | side_effects |
|---|---|---|---|---|---|---|
| PAYROLL_PREPARE_DRAFT | DRAFT | Payroll | Produce a payroll draft for employee + pay period | DRAFT | v1 | Write draft record only |
| PAYROLL_COMMIT_RUN | COMMIT | Payroll | Finalize payroll for a pay run | DRAFT | v1 | Commit payroll run (irreversible) |
| LINK_INVITE_GENERATE_DRAFT | DRAFT | Link | Create onboarding draft + minimal token preview (no send) | DRAFT | v1 | Write onboarding draft + token map only |
| LINK_INVITE_DRAFT_UPDATE_COMMIT | COMMIT | Link | Update onboarding draft with creator-provided fields (deterministic) | DRAFT | v1 | Update onboarding draft record only |
| LINK_INVITE_SEND_COMMIT | COMMIT | Link | Legacy do-not-wire send placeholder; delivery owned by LINK_DELIVER_INVITE | LEGACY_DO_NOT_WIRE | v1 | LEGACY_DO_NOT_WIRE; delivery only via LINK_DELIVER_INVITE |
| LINK_INVITE_OPEN_ACTIVATE_COMMIT | COMMIT | Link | Validate invite link on open, bind device fingerprint, enforce idempotency key, resolve token_id->draft_id | DRAFT | v1 | Mark link OPENED/ACTIVATED; mismatch executes single forward-block branch |
| LINK_INVITE_RESEND_COMMIT | COMMIT | Link | Legacy do-not-wire resend placeholder; delivery owned by LINK_DELIVER_INVITE | LEGACY_DO_NOT_WIRE | v1 | LEGACY_DO_NOT_WIRE; delivery only via LINK_DELIVER_INVITE |
| LINK_INVITE_REVOKE_REVOKE | REVOKE | Link | Revoke an invite link (activated/opened links require AP override; otherwise fail-closed) | DRAFT | v1 | Mark link REVOKED (state write) |
| LINK_INVITE_EXPIRED_RECOVERY_COMMIT | COMMIT | Link | Replace an expired invite link with a deterministic replacement preserving metadata | ACTIVE | v1 | State write only (replacement token/link_url); delivery via LINK_DELIVER_INVITE |
| LINK_INVITE_FORWARD_BLOCK_COMMIT | COMMIT | Link | Block forwarded-link activation on binding mismatch and emit escalation signals | DRAFT | v1 | Mark blocked attempt; audit + optional escalation |
| LINK_ROLE_PROPOSE_DRAFT | DRAFT | Link | Propose a new role/position template for AP approval (sandbox) | DRAFT | v1 | Write role proposal draft only |
| LINK_INVITE_DUAL_ROLE_CONFLICT_ESCALATE_DRAFT | DRAFT | Link | Detect dual-role conflict and open an escalation case (no execution) | DRAFT | v1 | Write escalation draft only |
| LINK_DELIVERY_FAILURE_HANDLING_COMMIT | COMMIT | Link | Legacy do-not-wire failure placeholder; delivery owned by LINK_DELIVER_INVITE | LEGACY_DO_NOT_WIRE | v1 | LEGACY_DO_NOT_WIRE; state handling only, delivery via LINK_DELIVER_INVITE |
| ONB_SESSION_START_DRAFT | DRAFT | Onboarding | Create onboarding session draft after link activation | DRAFT | v1 | Write onboarding session draft only |
| ONB_BIZ_START_DRAFT | DRAFT | OnboardingBusiness | Start company/tenant prerequisite onboarding draft | DRAFT | v1 | Write business onboarding draft only |
| ONB_BIZ_VALIDATE_COMPANY_COMMIT | COMMIT | OnboardingBusiness | Validate/create company shell prerequisites deterministically | DRAFT | v1 | Create/update tenant company shell |
| ONB_BIZ_COMPLETE_COMMIT | COMMIT | OnboardingBusiness | Finalize business onboarding prerequisite state | DRAFT | v1 | Mark business onboarding complete |
| ONB_DRAFT_UPDATE_COMMIT | COMMIT | Onboarding | Update onboarding draft with invitee answers and recompute missing_required_fields | DRAFT | v1 | Update onboarding draft record only |
| ONB_TERMS_ACCEPT_COMMIT | COMMIT | Onboarding | Record invitee Terms acceptance | DRAFT | v1 | Write terms acceptance record |
| ONB_EMPLOYEE_PHOTO_CAPTURE_SEND_COMMIT | COMMIT | Onboarding | Commit schema-required photo/evidence capture + sender handoff (legacy id retained) | DRAFT | v1 | Deliver evidence handoff via PH1.BCAST.001; write proof |
| ONB_EMPLOYEE_SENDER_VERIFY_COMMIT | COMMIT | Onboarding | Commit schema-required sender confirmation decision (legacy id retained) | DRAFT | v1 | Update verification state |
| ONB_PRIMARY_DEVICE_CONFIRM_COMMIT | COMMIT | Onboarding | Record primary device proof (biometric/passcode) | DRAFT | v1 | Write device proof record |
| ONB_ACCESS_INSTANCE_CREATE_COMMIT | COMMIT | Onboarding | Create per-user access instance (PH2.ACCESS.002 clone) | DRAFT | v1 | Create access instance record |
| ONB_COMPLETE_COMMIT | COMMIT | Onboarding | Atomically commit onboarding draft into real employee/user record and complete onboarding | DRAFT | v1 | Atomic commit + draft finalization + token consume + audit |
| ONB_REQUIREMENT_BACKFILL_START_DRAFT | DRAFT | Onboarding | Create deterministic campaign draft for applying new requirements to current staff | ACTIVE | v1 | Write backfill campaign draft + target snapshot |
| ONB_REQUIREMENT_BACKFILL_NOTIFY_COMMIT | COMMIT | Onboarding | Notify affected current staff to provide newly required fields/proofs | ACTIVE | v1 | Write campaign progress + broadcast/reminder handoff refs |
| ONB_REQUIREMENT_BACKFILL_COMPLETE_COMMIT | COMMIT | Onboarding | Finalize backfill campaign with deterministic completion/exception counts | ACTIVE | v1 | Write campaign terminal state |
| POSITION_SIM_001_CREATE_DRAFT | DRAFT | Position | Create deterministic position draft | DRAFT | v1 | Write position draft row |
| POSITION_SIM_002_VALIDATE_AUTH_COMPANY | DRAFT | Position | Validate authority scope and company existence before activation | DRAFT | v1 | Write validation result only |
| POSITION_SIM_003_BAND_POLICY_CHECK | DRAFT | Position | Apply compensation-band policy checks and escalation triggers | DRAFT | v1 | Write policy check result only |
| POSITION_SIM_004_ACTIVATE_COMMIT | COMMIT | Position | Activate a position draft | DRAFT | v1 | Update position lifecycle to Active |
| POSITION_SIM_005_RETIRE_OR_SUSPEND_COMMIT | COMMIT | Position | Retire or suspend an active position deterministically | DRAFT | v1 | Update position lifecycle state |
| POSITION_REQUIREMENTS_SCHEMA_CREATE_DRAFT | DRAFT | Position | Create requirements schema draft for a position | ACTIVE | v1 | Write position requirements schema draft |
| POSITION_REQUIREMENTS_SCHEMA_UPDATE_COMMIT | COMMIT | Position | Update requirements schema draft/overlays for a position | ACTIVE | v1 | Write requirements schema update |
| POSITION_REQUIREMENTS_SCHEMA_ACTIVATE_COMMIT | COMMIT | Position | Activate a position requirements schema version | ACTIVE | v1 | Update active schema version for position |
| VOICE_ID_ENROLL_START_DRAFT | DRAFT | VoiceIdentity | Start voice recognition enrollment loop (tight record->grade) | DRAFT | v1 | Write enrollment session state only |
| VOICE_ID_ENROLL_SAMPLE_COMMIT | COMMIT | VoiceIdentity | Capture + grade one voice enrollment sample and update progress | DRAFT | v1 | Store derived features + update enrollment progress |
| VOICE_ID_ENROLL_COMPLETE_COMMIT | COMMIT | VoiceIdentity | Finalize voice profile artifact after PASS target reached | DRAFT | v1 | Write voice profile artifact (derived; no raw audio) |
| VOICE_ID_ENROLL_DEFER_COMMIT | COMMIT | VoiceIdentity | Mark voice enrollment as PENDING and request reminder scheduling | DRAFT | v1 | Update enrollment state only |
| WAKE_ENROLL_START_DRAFT | DRAFT | Wake | Start wake-word enrollment loop (tight record->grade) | DRAFT | v1 | Write enrollment session state only |
| WAKE_ENROLL_SAMPLE_COMMIT | COMMIT | Wake | Capture + grade one wake enrollment sample and update progress | DRAFT | v1 | Store wake sample features + update enrollment progress |
| WAKE_ENROLL_COMPLETE_COMMIT | COMMIT | Wake | Finalize wake profile artifact after PASS target reached | DRAFT | v1 | Write wake profile artifact (derived; no raw audio) |
| WAKE_ENROLL_DEFER_COMMIT | COMMIT | Wake | Mark wake enrollment as PENDING and request reminder scheduling | DRAFT | v1 | Update enrollment state only |
| ACCESS_OVERRIDE_TEMP_GRANT_COMMIT | COMMIT | Access | Apply an AP-approved temporary/one-shot override to a user's per-user access instance | DRAFT | v1 | Update PH2.ACCESS.002 override state |
| ACCESS_OVERRIDE_PERM_GRANT_COMMIT | COMMIT | Access | Apply an AP-approved permanent override to a user's per-user access instance | DRAFT | v1 | Update PH2.ACCESS.002 baseline/override state |
| ACCESS_OVERRIDE_REVOKE_COMMIT | COMMIT | Access | Revoke a prior override from a user's per-user access instance | DRAFT | v1 | Update PH2.ACCESS.002 override state |
| ACCESS_AP_AUTHORING_REVIEW_CHANNEL_COMMIT | COMMIT | Access | Persist explicit AP authoring review-channel choice before schema lifecycle writes | DRAFT | v1 | Append AP authoring review state |
| ACCESS_AP_AUTHORING_RULE_ACTION_COMMIT | COMMIT | Access | Persist one bounded AP rule-review action (agree/disagree/edit/delete/disable/add custom) | DRAFT | v1 | Append AP authoring rule review action |
| ACCESS_AP_AUTHORING_CONFIRM_COMMIT | COMMIT | Access | Persist final AP authoring confirmation state before activation path | DRAFT | v1 | Append AP authoring confirmation state |
| ACCESS_AP_SCHEMA_CREATE_DRAFT | DRAFT | Access | Create AP schema draft row for global/tenant scope | DRAFT | v1 | Append AP schema ledger row only |
| ACCESS_AP_SCHEMA_UPDATE_COMMIT | COMMIT | Access | Update AP schema draft version deterministically | DRAFT | v1 | Append AP schema ledger row only |
| ACCESS_AP_SCHEMA_ACTIVATE_COMMIT | COMMIT | Access | Activate AP schema version into current projection | DRAFT | v1 | Update AP current projection |
| ACCESS_AP_SCHEMA_RETIRE_COMMIT | COMMIT | Access | Retire AP schema version (history retained) | DRAFT | v1 | Update AP current projection + ledger append |
| ACCESS_AP_OVERLAY_UPDATE_COMMIT | COMMIT | Access | Update tenant AP overlay operations and active projection | DRAFT | v1 | Append overlay ledger row + update current projection |
| ACCESS_BOARD_POLICY_UPDATE_COMMIT | COMMIT | Access | Update tenant board/approval policy definition and active projection | DRAFT | v1 | Append board policy ledger row + update current projection |
| ACCESS_BOARD_VOTE_COMMIT | COMMIT | Access | Record one board vote for an escalation case | DRAFT | v1 | Append board vote ledger row |
| ACCESS_INSTANCE_COMPILE_COMMIT | COMMIT | Access | Compile and persist per-user effective access lineage from schema chain | DRAFT | v1 | Upsert access instance lineage refs |
| CAPREQ_CREATE_DRAFT | DRAFT | CapabilityRequest | Create a deterministic capability request draft lifecycle row | ACTIVE | v1 | Append `capreq_ledger`; update `capreq_current` |
| CAPREQ_SUBMIT_FOR_APPROVAL_COMMIT | COMMIT | CapabilityRequest | Submit a capability request for approval | ACTIVE | v1 | Append `capreq_ledger`; update `capreq_current` |
| CAPREQ_APPROVE_COMMIT | COMMIT | CapabilityRequest | Approve a capability request deterministically | ACTIVE | v1 | Append `capreq_ledger`; update `capreq_current` |
| CAPREQ_REJECT_COMMIT | COMMIT | CapabilityRequest | Reject a capability request deterministically | ACTIVE | v1 | Append `capreq_ledger`; update `capreq_current` |
| CAPREQ_FULFILL_COMMIT | COMMIT | CapabilityRequest | Mark an approved capability request as fulfilled | ACTIVE | v1 | Append `capreq_ledger`; update `capreq_current` |
| CAPREQ_CANCEL_REVOKE | REVOKE | CapabilityRequest | Cancel an open capability request deterministically | ACTIVE | v1 | Append `capreq_ledger`; update `capreq_current` |
| REMINDER_SCHEDULE_COMMIT | COMMIT | Reminder | Schedule a reminder deterministically (timezone/DST/recurrence bounded) | ACTIVE | v1 | Write reminder record + occurrences |
| REMINDER_UPDATE_COMMIT | COMMIT | Reminder | Update reminder fields deterministically | ACTIVE | v1 | Update reminder record |
| REMINDER_CANCEL_COMMIT | COMMIT | Reminder | Cancel a reminder deterministically | ACTIVE | v1 | Update reminder state to CANCELED |
| REMINDER_SNOOZE_COMMIT | COMMIT | Reminder | Snooze a reminder by a bounded duration | ACTIVE | v1 | Update reminder state to SNOOZED |
| REMINDER_DELIVER_PRE_COMMIT | COMMIT | Reminder | Deliver a pre-reminder alert (if configured) | ACTIVE | v1 | Deliver via channel adapter; write proof |
| REMINDER_DELIVER_DUE_COMMIT | COMMIT | Reminder | Deliver the due reminder at scheduled_time (respect quiet hours policy) | ACTIVE | v1 | Deliver via channel adapter; write proof |
| REMINDER_FOLLOWUP_SCHEDULE_COMMIT | COMMIT | Reminder | Schedule a follow-up when no acknowledgment is received | ACTIVE | v1 | Write follow-up schedule state |
| REMINDER_ESCALATE_COMMIT | COMMIT | Reminder | Escalate reminder delivery to the next channel deterministically | ACTIVE | v1 | Deliver via escalated channel; write proof |
| REMINDER_DELIVERY_RETRY_SCHEDULE_COMMIT | COMMIT | Reminder | Schedule a bounded delivery retry after a delivery failure | ACTIVE | v1 | Write retry schedule state |
| REMINDER_MARK_COMPLETED_COMMIT | COMMIT | Reminder | Mark reminder occurrence as completed on user acknowledgment | ACTIVE | v1 | Update reminder occurrence state |
| REMINDER_MARK_FAILED_COMMIT | COMMIT | Reminder | Mark reminder occurrence as failed after retries/max attempts exhausted | ACTIVE | v1 | Update reminder occurrence state |
| EMO_SIM_001 | COMMIT | Emotional | Classify and lock the user's personality profile (tone-only) | ACTIVE | v1 | Update emotional profile record |
| EMO_SIM_002 | COMMIT | Emotional | Re-evaluate and update the user's personality profile at deterministic gates | ACTIVE | v1 | Update emotional profile record |
| EMO_SIM_003 | COMMIT | Emotional | Apply emotional privacy commands (forget/do-not-remember/recall-only/archive) | ACTIVE | v1 | Update emotional privacy policy state |
| EMO_SIM_004 | DRAFT | Emotional | Emit per-turn tone guidance (no profile mutation) | ACTIVE | v1 | No side effects (output only) |
| EMO_SIM_005 | COMMIT | Emotional | Capture an onboarding snapshot for emotional profile initialization | ACTIVE | v1 | Write snapshot record |
| EMO_SIM_006 | COMMIT | Emotional | Emit an audit-grade emotional event with reason codes | ACTIVE | v1 | Append audit event |
| BCAST-SIM-001 | DRAFT | Broadcast | Create a broadcast draft (immutable envelope after send) | DRAFT | v1 | Write broadcast draft only |
| BCAST-SIM-002 | DRAFT | Broadcast | Validate sender authority (classification/audience/channel) | DRAFT | v1 | Write validation result only |
| BCAST-SIM-003 | DRAFT | Broadcast | Resolve privacy handshake decision (OutLoud/DeviceOnly/Mixed) | DRAFT | v1 | Write recipient privacy decision only |
| BCAST-SIM-004 | COMMIT | Broadcast | Deliver to a recipient (handshake -> privacy -> content -> ack requirements) | DRAFT | v1 | Deliver via voice/push/in-app/email + proof |
| BCAST-SIM-005 | COMMIT | Broadcast | Defer and schedule retry/follow-up (handoff to Reminder Engine when needed) | DRAFT | v1 | Schedule retry/follow-up + state write |
| BCAST-SIM-006 | COMMIT | Broadcast | Record recipient acknowledgement (Read/Confirm/Action-Confirm) | DRAFT | v1 | Update recipient state to ACKNOWLEDGED |
| BCAST-SIM-007 | COMMIT | Broadcast | Escalate to sender with reason codes when blocked/unreachable | DRAFT | v1 | Deliver escalation + state write |
| BCAST-SIM-008 | COMMIT | Broadcast | Expire a broadcast (hard stop) | DRAFT | v1 | Mark expired + notify if required |
| BCAST_CREATE_DRAFT | DRAFT | Broadcast | Create canonical broadcast envelope draft used by PH1.BCAST lifecycle | DRAFT | v1 | Append draft lifecycle event + current projection update |
| BCAST_DELIVER_COMMIT | COMMIT | Broadcast | Commit one recipient delivery request from approved broadcast envelope | DRAFT | v1 | Append delivery attempt + recipient state transition |
| BCAST_DEFER_COMMIT | COMMIT | Broadcast | Commit recipient defer decision and deterministic retry schedule | DRAFT | v1 | Append defer lifecycle event + retry schedule update |
| BCAST_REMINDER_FIRED_COMMIT | COMMIT | Broadcast | Commit BCAST.MHP reminder-fire lifecycle resume (`REMINDER_SET -> REMINDER_FIRED`) | DRAFT | v1 | Append reminder-fired lifecycle event + recipient state transition |
| BCAST_ACK_COMMIT | COMMIT | Broadcast | Commit recipient acknowledgement (READ/CONFIRM/ACTION_CONFIRM) | DRAFT | v1 | Append ack lifecycle event + recipient state transition |
| BCAST_ESCALATE_COMMIT | COMMIT | Broadcast | Commit escalation-to-sender lifecycle update for blocked/unreachable recipients | DRAFT | v1 | Append escalation lifecycle event |
| BCAST_EXPIRE_COMMIT | COMMIT | Broadcast | Commit broadcast expiry and close unresolved recipient states | DRAFT | v1 | Append expire lifecycle event + envelope close |
| BCAST_CANCEL_COMMIT | COMMIT | Broadcast | Commit broadcast cancellation by sender/policy | DRAFT | v1 | Append cancel lifecycle event + envelope close |
| DELIVERY_SEND_COMMIT | COMMIT | Delivery | Commit provider send request for a prepared recipient payload | DRAFT | v1 | Append delivery provider attempt + proof reference |
| DELIVERY_CANCEL_COMMIT | COMMIT | Delivery | Commit provider cancel request for an in-flight delivery attempt | DRAFT | v1 | Append provider cancel attempt + status update |
| SMS_SETUP_SIM | COMMIT | OnboardingSms | Commit SMS app setup completion state for a user | DRAFT | v1 | Append SMS setup lifecycle event + current projection update |
| LEARN_MODEL_UPDATE_SIM | COMMIT | Learning | Commit governed learning artifact updates from feedback/correction loops | DRAFT | v1 | Append adaptation artifact event rows |
| MEMORY_FORGET_COMMIT | COMMIT | Memory | Commit memory forget request with deterministic bounded scope | ACTIVE | v1 | Append forget event + update memory current projection |
| MEMORY_SUPPRESSION_SET_COMMIT | COMMIT | Memory | Commit memory suppression control update (`DO_NOT_MENTION|DO_NOT_REPEAT|DO_NOT_STORE`) | ACTIVE | v1 | Upsert suppression rule state |
| MEMORY_ATOM_UPSERT_COMMIT | COMMIT | Memory | Commit one memory atom store/update event deterministically | ACTIVE | v1 | Append atom event + update atom current projection |
| TOOL_TIME_QUERY_COMMIT | COMMIT | Tool | Commit read-only TIME query outcome as PH1.E audit row | ACTIVE | v1 | Append ToolOk/ToolFail audit row only |
| TOOL_WEATHER_QUERY_COMMIT | COMMIT | Tool | Commit read-only WEATHER query outcome as PH1.E audit row | ACTIVE | v1 | Append ToolOk/ToolFail audit row only |
| TENANT_CONTEXT_RESOLVE_DRAFT | DRAFT | Tenant | Resolve tenant and policy context before enterprise execution | DRAFT | v1 | Write context resolution result only |
| QUOTA_CHECK_DRAFT | DRAFT | Quota | Evaluate deterministic budget/quota gates for a request | DRAFT | v1 | Write quota decision result only |
| KMS_HANDLE_ISSUE_COMMIT | COMMIT | KMS | Issue short-lived credential handle for approved runtime use | DRAFT | v1 | Write handle issue/audit row |
| WORK_ORDER_APPEND_COMMIT | COMMIT | WorkOrder | Append deterministic work order event to ledger | DRAFT | v1 | Append work_order_ledger row |
| WORK_LEASE_ACQUIRE_COMMIT | COMMIT | WorkLease | Acquire deterministic lease for a work order | DRAFT | v1 | Write work_order_leases row |
| WORK_LEASE_RENEW_COMMIT | COMMIT | WorkLease | Renew deterministic lease for a work order | DRAFT | v1 | Update work_order_leases row |
| WORK_LEASE_RELEASE_COMMIT | COMMIT | WorkLease | Release deterministic lease for a work order | DRAFT | v1 | Update work_order_leases row |
| SCHED_NEXT_ACTION_DRAFT | DRAFT | Scheduler | Compute deterministic retry/timeout next action | DRAFT | v1 | Write scheduler decision result only |
| GOV_ACTIVATE_DEFINITION_COMMIT | COMMIT | Governance | Activate a signed blueprint/simulation definition set | DRAFT | v1 | Update governance_definitions state |
| GOV_ROLLBACK_DEFINITION_COMMIT | COMMIT | Governance | Roll back active definition set to prior version | DRAFT | v1 | Update governance_definitions state |
| EXPORT_BUILD_PACK_COMMIT | COMMIT | Export | Build tamper-evident compliance export pack | DRAFT | v1 | Write export_jobs row + artifact ref |
| REVIEW_ROUTE_CASE_DRAFT | DRAFT | Governance | Route a policy-required human review case through governance flow | DRAFT | v1 | Write review_cases draft row |
| REVIEW_DECISION_COMMIT | COMMIT | Governance | Commit human review decision deterministically through governance flow | DRAFT | v1 | Update review_cases decision state |

## Domain DB Binding Profiles (Authoritative)

Hard rules
- Every simulation record must use its owning domain profile below unless it declares a stricter record-level override.
- Record-level overrides are allowed only when they reduce scope, never when they widen scope.
- Simulations with missing profile/override bindings stay `DRAFT` and cannot become `ACTIVE`.

| owning_domain | reads_tables[] | writes_tables[] |
|---|---|---|
| Payroll | [`identities`, `tenant_companies`, `positions`] | [`artifacts_ledger`] |
| Link | [`onboarding_drafts`, `onboarding_link_tokens`, `positions`, `tenant_companies`] | [`onboarding_drafts`, `onboarding_link_tokens`, `onboarding_draft_write_dedupe`] |
| Onboarding | [`onboarding_drafts`, `onboarding_link_tokens`, `tenant_companies`, `positions`, `access_instances`, `access_overrides`, `onboarding_backfill_campaigns`] | [`onboarding_drafts`, `onboarding_link_tokens`, `onboarding_draft_write_dedupe`, `access_instances`, `onboarding_backfill_campaigns`] |
| OnboardingBusiness | [`tenant_companies`] | [`tenant_companies`] |
| Position | [`tenant_companies`, `positions`, `position_requirements_schemas_current`] | [`positions`, `position_lifecycle_events`, `position_requirements_schemas_ledger`, `position_requirements_schemas_current`] |
| VoiceIdentity | [`identities`, `devices`, `sessions`] | [`artifacts_ledger`] |
| Wake | [`devices`, `sessions`, `wake_enrollment_sessions`, `wake_profile_bindings`] | [`wake_enrollment_sessions`, `wake_enrollment_samples`, `wake_runtime_events`, `wake_profile_bindings`, `artifacts_ledger`] |
| Access | [`access_instances`, `access_overrides`, `access_ap_schemas_ledger`, `access_ap_schemas_current`, `access_ap_overlay_ledger`, `access_ap_overlay_current`, `access_board_policy_ledger`, `access_board_policy_current`, `access_board_votes_ledger`] | [`access_instances`, `access_overrides`, `access_ap_schemas_ledger`, `access_ap_schemas_current`, `access_ap_overlay_ledger`, `access_ap_overlay_current`, `access_board_policy_ledger`, `access_board_policy_current`, `access_board_votes_ledger`] |
| CapabilityRequest | [`access_instances`, `access_overrides`, `capreq_current`] | [`capreq_ledger`, `capreq_current`] |
| Reminder | [`reminders`, `reminder_occurrences`] | [`reminders`, `reminder_occurrences`, `reminder_delivery_attempts`] |
| Emotional | [`preferences_current`, `memory_current`] | [`preferences_ledger`, `artifacts_ledger`, `audit_events`] |
| Broadcast | [`comms.broadcast_envelopes_current`, `comms.broadcast_recipients_current`] | [`comms.broadcast_envelopes_ledger`, `comms.broadcast_envelopes_current`, `comms.broadcast_recipients_current`, `comms.broadcast_delivery_attempts_ledger`, `comms.broadcast_ack_ledger`] |
| Delivery | [`comms.delivery_attempts_current`, `comms.delivery_provider_health`] | [`comms.delivery_attempts_ledger`, `comms.delivery_attempts_current`] |
| OnboardingSms | [`comms.sms_app_setup_current`] | [`comms.sms_app_setup_ledger`, `comms.sms_app_setup_current`] |
| Language | [`conversation_ledger`] | [`audit_events`] |
| Learning | [`artifacts_ledger`] | [`artifacts_ledger`] |
| Memory | [`memory.memory_atoms_current`, `memory.memory_suppression_rules`, `memory.memory_threads_current`] | [`memory.memory_atoms_ledger`, `memory.memory_atoms_current`, `memory.memory_suppression_rules`, `memory.memory_threads_current`] |
| Tool | [`identities`, `devices`, `sessions`, `audit_events`] | [`audit_events`] |
| Tenant | [`tenant_companies`] | [`tenant_companies`] |
| Quota | [`artifacts_ledger`] | [`artifacts_ledger`] |
| KMS | [`governance_definitions`] | [`governance_definitions`] |
| WorkOrder | [`work_order_ledger`] | [`work_order_ledger`] |
| WorkLease | [`work_order_ledger`, `work_order_leases`] | [`work_order_leases`] |
| Scheduler | [`work_order_ledger`, `work_order_leases`] | [`work_order_ledger`] |
| Governance | [`governance_definitions`, `review_cases`, `work_order_ledger`] | [`governance_definitions`, `review_cases`] |
| Export | [`audit_events`, `conversation_ledger`, `work_order_ledger`] | [`export_jobs`, `artifacts_ledger`] |

## Contract Completeness Defaults (No TBD Allowed)

These defaults apply when a record does not provide a stricter value:
- required_roles: `POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)`
- preconditions: `input schema valid + tenant boundary valid + blueprint references simulation + access/confirmation gates satisfied where required`
- idempotency_key_rule: `required for retriable writes; dedupe on (simulation_id + business_key + idempotency_key)`
- audit_events: `[SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]`

## Simulation Records (Detailed)

### PAYROLL_PREPARE_DRAFT (DRAFT)

- name: Payroll Prepare Draft
- owning_domain: Payroll
- simulation_type: DRAFT
- purpose: Produce a payroll draft for employee + pay period
- triggers: PAYROLL_PREPARE (process_id) (example)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: none (example)
- required_confirmations: none (DRAFT)
- input_schema (minimum):
```text
employee_id: string
pay_period: string
```
- output_schema (minimum):
```text
payroll_draft_id: string
gross_pay: number
deductions: number
net_pay: number
```
- preconditions: input schema valid + tenant boundary valid + blueprint references simulation + access/confirmation gates satisfied where required
- postconditions: Draft exists; no irreversible changes
- side_effects: Write draft record only
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: required for retriable writes; dedupe on (simulation_id + business_key + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### PAYROLL_COMMIT_RUN (COMMIT)

- name: Payroll Commit Run
- owning_domain: Payroll
- simulation_type: COMMIT
- purpose: Finalize payroll for a pay run
- triggers: PAYROLL_RUN (process_id) (example)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: optional AP approval (example)
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
payroll_draft_id: string
confirmation_token: string
```
- output_schema (minimum):
```text
payroll_run_id: string
status: COMMITTED
```
- preconditions (minimum): Access approved; confirmation received
- postconditions: Payroll run is committed (irreversible without compensating simulation)
- side_effects: Commit payroll run (irreversible)
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: required for retriable writes; dedupe on (simulation_id + business_key + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### LINK_INVITE_GENERATE_DRAFT (DRAFT)

- name: Link Invite Generate (Draft)
- owning_domain: Link
- simulation_type: DRAFT
- purpose: Create a deterministic onboarding draft in PH1.F and issue a minimal invite token preview (no delivery yet)
- triggers: LINK_INVITE (process_id) (example)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: none (example)
- required_confirmations: none (DRAFT)
- input_schema (minimum):
```text
inviter_user_id: string
invitee_type: enum (COMPANY | CUSTOMER | EMPLOYEE | FAMILY_MEMBER | FRIEND | ASSOCIATE)
tenant_id: string (optional; required for EMPLOYEE)
schema_version_id: string (required for EMPLOYEE and COMPANY)
creator_prefilled_fields: object (bounded; optional)
expiration_policy_id: string (optional)
```
- output_schema (minimum):
```text
draft_id: string
token_id: string
link_url: string
payload_hash: string
expires_at: timestamp_ms
status: DRAFT_CREATED
missing_required_fields: string[]
```
- preconditions: Identity OK; required fields present; policy snapshot available; schema exists when invitee_type in (EMPLOYEE, COMPANY)
- postconditions: onboarding_draft row exists; token row exists; mapping `token_id -> draft_id` exists
- side_effects: Write onboarding draft + token mapping only
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (inviter_user_id + invitee_type + tenant_id + payload_hash + schema_version_id + expiration_policy_id)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### LINK_INVITE_DRAFT_UPDATE_COMMIT (COMMIT)

- name: Link Invite Draft Update (Commit)
- owning_domain: Link
- simulation_type: COMMIT
- purpose: Update onboarding draft fields from creator input and recompute missing_required_fields deterministically
- triggers: LINK_INVITE_EDIT (process_id) (example)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: optional AP approval (policy-dependent for sensitive fields)
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
draft_id: string
creator_update_fields: object (bounded)
idempotency_key: string
```
- output_schema (minimum):
```text
draft_id: string
missing_required_fields: string[]
draft_status: enum (DRAFT_CREATED | DRAFT_READY)
```
- preconditions: draft exists; draft is not terminal (`COMMITTED|REVOKED|EXPIRED`); linked token state is not terminal (`CONSUMED|REVOKED|EXPIRED`)
- postconditions: same draft updated; missing_required_fields recomputed from active tenant schema version + selector snapshot only
- side_effects: Update onboarding draft record only
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (draft_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### LINK_INVITE_SEND_COMMIT (COMMIT)

LEGACY_DO_NOT_WIRE: Delivery is performed only by LINK_DELIVER_INVITE (PH1.BCAST + PH1.DELIVERY). This simulation must not be wired.

- name: Link Invite Send (Commit)
- owning_domain: Link
- simulation_type: COMMIT
- purpose: Deliver the invite link via the selected channel and record proof
- triggers: LINK_INVITE (process_id) (example)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: optional AP approval (policy-dependent)
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
token_id: string
draft_id: string
delivery_method: enum (SMS | Email | WhatsApp | WeChat | QR | CopyLink)
recipient_contact: string
idempotency_key: string
```
- output_schema (minimum):
```text
token_id: string
delivery_status: enum (SENT | FAIL)
delivery_proof_ref: string (optional)
status: SENT
```
- preconditions (minimum): token exists; token maps to draft_id; draft not COMMITTED; not expired/revoked; Access approved; confirmation received
- postconditions: Delivery proof exists (or fail reason is recorded deterministically)
- side_effects: Send via delivery channel; write proof record
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (token_id + delivery_method + recipient_contact + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### LINK_INVITE_OPEN_ACTIVATE_COMMIT (COMMIT)

- name: Link Invite Open + Activate (Commit)
- owning_domain: Link
- simulation_type: COMMIT
- purpose: Validate invite token on open, bind device fingerprint, resolve token_id->draft_id
- triggers: LINK_OPEN (process_id) (example)
- required_roles: none (invitee-side)
- required_approvals: none (example)
- required_confirmations: none
- input_schema (minimum):
```text
token_id: string
device_fingerprint: string
idempotency_key: string
now_ms: timestamp_ms
```
- output_schema (minimum):
```text
token_id: string
draft_id: string (optional)
activation_status: enum (ACTIVATED | BLOCKED | EXPIRED | REVOKED | CONSUMED)
bound_device_fingerprint_hash: string (optional)
missing_required_fields: string[] (optional)
```
- preconditions (minimum): token_id exists; token maps to draft_id; if expired -> EXPIRED; if revoked -> REVOKED; if already consumed -> CONSUMED; if binding mismatch -> execute LINK_INVITE_FORWARD_BLOCK_COMMIT branch and return BLOCKED
- postconditions: first-open binds device fingerprint hash; onboarding handoff carries draft_id and current missing_required_fields; mismatch branch records one deterministic BLOCKED write
- side_effects: State write only (`DRAFT_CREATED|SENT -> OPENED -> ACTIVATED` internal transition + bindings; terminal passthrough for CONSUMED/EXPIRED/REVOKED/BLOCKED)
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (token_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### LINK_INVITE_RESEND_COMMIT (COMMIT)

LEGACY_DO_NOT_WIRE: Delivery is performed only by LINK_DELIVER_INVITE (PH1.BCAST + PH1.DELIVERY). This simulation must not be wired.

- name: Link Invite Resend (Commit)
- owning_domain: Link
- simulation_type: COMMIT
- purpose: Resend an existing invite link idempotently
- triggers: LINK_RESEND (process_id) (example)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: optional AP approval (policy-dependent)
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
token_id: string
delivery_method: enum (SMS | Email | WhatsApp | WeChat)
recipient_contact: string
idempotency_key: string
```
- output_schema (minimum):
```text
token_id: string
delivery_status: enum (SENT | FAIL)
delivery_proof_ref: string (optional)
```
- preconditions: token_id exists; not revoked; policy allows resend
- postconditions: Delivery proof exists (or fail reason recorded)
- side_effects: Send via delivery channel; write proof record
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (token_id + delivery_method + recipient_contact + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### LINK_INVITE_REVOKE_REVOKE (REVOKE)

- name: Link Invite Revoke (Revoke)
- owning_domain: Link
- simulation_type: REVOKE
- purpose: Revoke an invite link (fail closed if already activated/opened without AP override)
- triggers: LINK_REVOKE (process_id) (example)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: AP override required when token is already `ACTIVATED` (or `OPENED`)
- required_confirmations: required (REVOKE)
- input_schema (minimum):
```text
token_id: string
reason: string (bounded)
```
- output_schema (minimum):
```text
token_id: string
status: REVOKED
```
- preconditions: token exists; if already activated/opened -> require AP override or refuse
- postconditions: token is revoked and cannot be used again
- side_effects: State write only (REVOKED)
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (token_id)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### LINK_INVITE_EXPIRED_RECOVERY_COMMIT (COMMIT)

- name: Link Invite Expired Recovery (Commit)
- owning_domain: Link
- simulation_type: COMMIT
- purpose: Replace an expired invite link with a deterministic replacement token/link_url preserving metadata; delivery is performed only by LINK_DELIVER_INVITE (PH1.BCAST + PH1.DELIVERY)
- triggers: LINK_EXPIRED_RECOVERY (process_id) (example)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: optional AP approval (policy-dependent)
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
expired_token_id: string
idempotency_key: string
```
- output_schema (minimum):
```text
new_token_id: string
new_link_url: string
```
- preconditions: expired_token_id exists and is expired; policy allows recovery
- postconditions: new token exists; old token remains expired (history preserved)
- side_effects: State write only (replacement token/link_url). Delivery is performed only by LINK_DELIVER_INVITE.
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (expired_token_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### LINK_INVITE_FORWARD_BLOCK_COMMIT (COMMIT)

- name: Link Invite Forward Block (Commit)
- owning_domain: Link
- simulation_type: COMMIT
- purpose: Block forwarded-link activation on binding mismatch and emit escalation signals
- triggers: LINK_OPEN_ACTIVATE mismatch branch (process_id)
- required_roles: none (invitee-side)
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
token_id: string
presented_device_fingerprint: string
```
- output_schema (minimum):
```text
token_id: string
activation_status: BLOCKED
reason_code: FORWARDED_LINK_BLOCKED
```
- preconditions: token exists and is already bound to a different device fingerprint; this is the single mismatch branch under LINK_INVITE_OPEN_ACTIVATE_COMMIT
- postconditions: blocked attempt is recorded; optional escalation case opened (policy-dependent)
- side_effects: State write only (blocked attempt record)
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (token_id + presented_device_fingerprint_hash)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### LINK_ROLE_PROPOSE_DRAFT (DRAFT)

- name: Link Role Proposal (Draft)
- owning_domain: Link
- simulation_type: DRAFT
- purpose: Propose a new role/position template for AP approval (sandbox)
- triggers: LINK_INVITE (process_id) (example)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: AP approval required to activate
- required_confirmations: required (DRAFT)
- input_schema (minimum):
```text
tenant_id: string
requested_role_name: string
requested_position_template: object (bounded)
```
- output_schema (minimum):
```text
role_proposal_id: string
status: PENDING_AP_APPROVAL
```
- preconditions: role does not already exist; requester authorized to propose
- postconditions: proposal exists; no role is activated yet
- side_effects: Write role proposal draft only
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (tenant_id + requested_role_name)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### LINK_INVITE_DUAL_ROLE_CONFLICT_ESCALATE_DRAFT (DRAFT)

- name: Link Dual Role Conflict Escalation (Draft)
- owning_domain: Link
- simulation_type: DRAFT
- purpose: Detect dual-role conflict and open an escalation case (no execution)
- triggers: LINK_INVITE (process_id) (example)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: optional AP approval (policy-dependent)
- required_confirmations: required (DRAFT)
- input_schema (minimum):
```text
tenant_id: string
invitee_identity_hint: string (bounded)
conflict_reason: string (bounded)
```
- output_schema (minimum):
```text
escalation_case_id: string
status: ESCALATED
```
- preconditions: conflict detected deterministically (existing role/identity mismatch)
- postconditions: escalation case exists; no invite is sent
- side_effects: Write escalation draft only
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (tenant_id + invitee_identity_hint + conflict_reason)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### LINK_DELIVERY_FAILURE_HANDLING_COMMIT (COMMIT)

LEGACY_DO_NOT_WIRE: Delivery is performed only by LINK_DELIVER_INVITE (PH1.BCAST + PH1.DELIVERY). This simulation must not be wired.

- name: Link Delivery Failure Handling (Commit)
- owning_domain: Link
- simulation_type: COMMIT
- purpose: Deterministic delivery failure recovery (retry/alternate channel)
- triggers: LINK_DELIVERY_FAIL (process_id) (example)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: none (example)
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
token_id: string
last_delivery_error: string (bounded)
retry_method: enum (RETRY_SAME_CHANNEL | SWITCH_CHANNEL)
alternate_delivery_method: enum (SMS | Email | WhatsApp | WeChat) (optional)
idempotency_key: string
```
- output_schema (minimum):
```text
token_id: string
delivery_status: enum (SENT | FAIL)
delivery_proof_ref: string (optional)
```
- preconditions: token exists; policy allows retry/switch
- postconditions: delivery attempt is recorded deterministically; no duplicate sends on retry
- side_effects: Send retry or alternate channel; write proof record
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (token_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### ONB_SESSION_START_DRAFT (DRAFT)

- name: Onboarding Session Start (Draft)
- owning_domain: Onboarding
- simulation_type: DRAFT
- purpose: Create an onboarding session draft after a link is ACTIVATED (invitee-side)
- triggers: LINK_OPEN_ACTIVATED (example)
- required_roles: none (invitee-side)
- required_approvals: none
- required_confirmations: none (DRAFT)
- input_schema (minimum):
```text
token_id: string
tenant_id: string (optional; required for employee)
device_fingerprint: string
```
- output_schema (minimum):
```text
onboarding_session_id: string
status: DRAFT_CREATED
next_step: enum (INSTALL | TERMS | LOAD_PREFILLED | ASK_MISSING)
```
- preconditions: token is ACTIVATED and not revoked
- postconditions: onboarding session draft exists with deterministic pinned schema context snapshot
- side_effects: Write onboarding session draft only
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (token_id)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### ONB_DRAFT_UPDATE_COMMIT (COMMIT)

- name: Onboarding Draft Update (Commit)
- owning_domain: Onboarding
- simulation_type: COMMIT
- purpose: Update onboarding draft with invitee answers and recompute missing_required_fields deterministically
- triggers: ONBOARDING_FIELD_CAPTURED (example)
- required_roles: none (invitee-side)
- required_approvals: none
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
onboarding_session_id: string
draft_id: string
invitee_update_fields: object (bounded)
idempotency_key: string
```
- output_schema (minimum):
```text
onboarding_session_id: string
draft_id: string
missing_required_fields: string[]
```
- preconditions: onboarding session exists; draft exists; draft is not COMMITTED/REVOKED
- postconditions: same draft updated; missing_required_fields recomputed from tenant schema version only
- side_effects: Update onboarding draft record only
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (onboarding_session_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### ONB_TERMS_ACCEPT_COMMIT (COMMIT)

- name: Onboarding Terms Accept (Commit)
- owning_domain: Onboarding
- simulation_type: COMMIT
- purpose: Record invitee Terms acceptance (or refusal) deterministically
- triggers: TERMS_ACCEPT (example)
- required_roles: none (invitee-side)
- required_approvals: none
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
onboarding_session_id: string
terms_version_id: string
accepted: bool
idempotency_key: string
```
- output_schema (minimum):
```text
onboarding_session_id: string
terms_status: enum (ACCEPTED | DECLINED)
```
- preconditions: onboarding_session exists
- postconditions: terms acceptance is recorded; decline ends onboarding
- side_effects: Write terms acceptance record only
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (onboarding_session_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### ONB_EMPLOYEE_PHOTO_CAPTURE_SEND_COMMIT (COMMIT)

- name: Schema-Required Evidence Capture + Sender Handoff (Commit; legacy id retained)
- owning_domain: Onboarding
- simulation_type: COMMIT
- purpose: Capture schema-required photo/evidence and send to designated confirmer when pinned schema requires sender confirmation
- triggers: SCHEMA_REQUIRED_EVIDENCE_CAPTURED (example)
- required_roles: none (invitee-side)
- required_approvals: none
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
onboarding_session_id: string
photo_blob_ref: string
sender_user_id: string
idempotency_key: string
```
- output_schema (minimum):
```text
onboarding_session_id: string
photo_proof_ref: string
verification_status: enum (PENDING)
```
- preconditions: pinned schema requires this evidence + sender confirmation; terms accepted
- postconditions: evidence is delivered to designated confirmer; verification state becomes PENDING; follow-up is required until confirmer CONFIRMS or REJECTS
- side_effects: Deliver evidence via PH1.BCAST.001 (Selene app) as Private/Confidential with required_ack=Action-Confirm; write proof record; schedule retries via PH1.REM.001 if not acknowledged
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (onboarding_session_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### ONB_EMPLOYEE_SENDER_VERIFY_COMMIT (COMMIT)

- name: Schema-Required Sender Verify (Commit; legacy id retained)
- owning_domain: Onboarding
- simulation_type: COMMIT
- purpose: Designated confirmer confirms or rejects schema-required evidence deterministically
- triggers: SCHEMA_REQUIRED_VERIFY_DECISION (example)
- required_roles: sender must have authority (policy)
- required_approvals: optional AP approval (policy-dependent)
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
onboarding_session_id: string
sender_user_id: string
decision: enum (CONFIRM | REJECT)
idempotency_key: string
```
- output_schema (minimum):
```text
onboarding_session_id: string
verification_status: enum (CONFIRMED | REJECTED)
```
- preconditions: schema-required verification is PENDING
- postconditions: gated completion/access may unlock only after CONFIRMED (when schema requires sender confirmation)
- side_effects: Update onboarding verification state only
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (onboarding_session_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### ONB_PRIMARY_DEVICE_CONFIRM_COMMIT (COMMIT)

- name: Primary Device Confirm (Commit)
- owning_domain: Onboarding
- simulation_type: COMMIT
- purpose: Record primary device proof success (biometric/passcode) deterministically
- triggers: DEVICE_PROOF_COLLECTED (example)
- required_roles: none (invitee-side)
- required_approvals: none
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
onboarding_session_id: string
device_id: string
proof_type: enum (BIOMETRIC | PASSCODE)
proof_ok: bool
idempotency_key: string
```
- output_schema (minimum):
```text
onboarding_session_id: string
primary_device_confirmed: bool
```
- preconditions: terms accepted
- postconditions: device confirmation recorded (still sandboxed until sender verifies if required)
- side_effects: Write device proof record only
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (onboarding_session_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### ONB_ACCESS_INSTANCE_CREATE_COMMIT (COMMIT)

- name: Access Instance Create (Commit)
- owning_domain: Onboarding
- simulation_type: COMMIT
- purpose: Create the per-user access instance (PH2.ACCESS.002 clone) for this onboarding session
- triggers: ACCESS_INSTANCE_CREATE (example)
- required_roles: onboarding authority (policy)
- required_approvals: optional AP approval (policy-dependent)
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
onboarding_session_id: string
user_id: string
tenant_id: string (optional)
role_id: string
idempotency_key: string
```
- output_schema (minimum):
```text
onboarding_session_id: string
access_engine_instance_id: string
```
- preconditions: identity + device proof gates complete per blueprint
- postconditions: per-user access instance exists (still sandboxed until blueprint allows activation)
- side_effects: Create access instance record (simulation-gated)
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (user_id + role_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### ONB_COMPLETE_COMMIT (COMMIT)

- name: Onboarding Complete (Commit)
- owning_domain: Onboarding
- simulation_type: COMMIT
- purpose: Atomically create the real employee/user record from onboarding draft, finalize draft, consume token, and emit final proof/audit
- triggers: ONBOARDING_COMPLETE (example)
- required_roles: onboarding authority (policy)
- required_approvals: none (example)
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
onboarding_session_id: string
draft_id: string
token_id: string
idempotency_key: string
```
- output_schema (minimum):
```text
onboarding_session_id: string
entity_id: string
onboarding_status: COMPLETE
draft_status: COMMITTED
token_status: CONSUMED
```
- preconditions: required gates complete per blueprint; `missing_required_fields` is empty; if invitee_type=EMPLOYEE then sender verification must be CONFIRMED (or an explicitly approved alternative path exists)
- postconditions: onboarding is COMPLETE; draft is COMMITTED; token is consumed/invalidated; no silent success
- side_effects: atomic state write (entity create + draft finalize + token consume) + audit emission
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (onboarding_session_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### ONB_REQUIREMENT_BACKFILL_START_DRAFT (DRAFT)

- name: Onboarding Requirement Backfill Start Draft
- owning_domain: Onboarding
- simulation_type: DRAFT
- purpose: Start deterministic backfill campaign when a new active requirement schema must be applied to current staff
- triggers: ONB_REQUIREMENT_BACKFILL (process_id)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: optional AP approval (policy-dependent)
- required_confirmations: required (rollout scope confirmation)
- input_schema (minimum):
```text
tenant_id: string
actor_user_id: string
company_id: string
position_id: string
schema_version_id: string
rollout_scope: enum (CurrentAndNew)
idempotency_key: string
```
- output_schema (minimum):
```text
campaign_id: string
state: enum (DRAFT_CREATED | RUNNING | COMPLETED)
pending_target_count: integer
```
- preconditions: active requirements schema exists for position; rollout scope confirmed; access decision is ALLOW/ESCALATE-resolved
- postconditions: campaign draft and deterministic target snapshot are persisted
- side_effects: Write backfill campaign draft + target snapshot only
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (tenant_id + position_id + schema_version_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### ONB_REQUIREMENT_BACKFILL_NOTIFY_COMMIT (COMMIT)

- name: Onboarding Requirement Backfill Notify Commit
- owning_domain: Onboarding
- simulation_type: COMMIT
- purpose: Commit campaign notification/progress state while issuing deterministic handoff refs for BCAST and REM follow-up
- triggers: ONB_REQUIREMENT_BACKFILL (process_id)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: none
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
campaign_id: string
tenant_id: string
recipient_user_id: string
idempotency_key: string
```
- output_schema (minimum):
```text
campaign_id: string
recipient_user_id: string
target_status: enum (PENDING | REQUESTED | REMINDED | COMPLETED | EXEMPTED | FAILED)
```
- preconditions: campaign exists and is IN_PROGRESS; recipient belongs to campaign target set
- postconditions: notification/progress state persists deterministically; no duplicate notify rows on retries
- side_effects: Write campaign/target progress state for this recipient after BCAST/REM handoff steps
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (campaign_id + recipient_user_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### ONB_REQUIREMENT_BACKFILL_COMPLETE_COMMIT (COMMIT)

- name: Onboarding Requirement Backfill Complete Commit
- owning_domain: Onboarding
- simulation_type: COMMIT
- purpose: Finalize backfill campaign with deterministic completion and unresolved exception counts
- triggers: ONB_REQUIREMENT_BACKFILL (process_id)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: none
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
campaign_id: string
tenant_id: string
idempotency_key: string
```
- output_schema (minimum):
```text
campaign_id: string
state: enum (COMPLETED)
completed_target_count: integer
total_target_count: integer
```
- preconditions: campaign exists; completion criteria evaluated deterministically
- postconditions: campaign terminal state is persisted and replay-stable
- side_effects: Write campaign terminal state
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (campaign_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### VOICE_ID_ENROLL_START_DRAFT (DRAFT)

- name: Voice ID Enroll Start (Draft)
- owning_domain: VoiceIdentity
- simulation_type: DRAFT
- purpose: Start the voice recognition enrollment loop (tight record -> grade -> next)
- triggers: ONB_VOICE_ENROLL_START (example)
- required_roles: none (invitee-side)
- required_approvals: none
- required_confirmations: none (DRAFT)
- input_schema (minimum):
```text
onboarding_session_id: string
device_id: string
consent_asserted: bool
now_ms: timestamp_ms
```
- output_schema (minimum):
```text
onboarding_session_id: string
voice_enroll_status: enum (IN_PROGRESS)
max_total_attempts: number (default 8, allowed 5 to 20)
max_session_enroll_time_ms: number (default 120000, allowed 60000 to 300000)
lock_after_consecutive_passes: number (default 3, allowed 2 to 5)
```
- preconditions: onboarding_session exists; terms accepted; consent_asserted=true
- postconditions: enrollment loop is ready for sample capture
- side_effects: Write enrollment session state only
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (onboarding_session_id + device_id)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### VOICE_ID_ENROLL_SAMPLE_COMMIT (COMMIT)

- name: Voice ID Enroll Sample (Commit)
- owning_domain: VoiceIdentity
- simulation_type: COMMIT
- purpose: Capture + grade one voice enrollment sample and update progress (reason-coded; no raw audio persisted by default)
- triggers: ONB_VOICE_ENROLL_SAMPLE (example)
- required_roles: none (invitee-side)
- required_approvals: none
- required_confirmations: none (consent gate is the precondition)
- input_schema (minimum):
```text
onboarding_session_id: string
audio_sample_ref: string (bounded)
attempt_index: number
now_ms: timestamp_ms
idempotency_key: string
```
- output_schema (minimum):
```text
onboarding_session_id: string
sample_result: enum (PASS | FAIL)
reason_code: string (optional; required on FAIL)
consecutive_passes: number
voice_enroll_status: enum (IN_PROGRESS | LOCKED | PENDING)
```
- preconditions: enrollment started; consent_asserted=true
- postconditions: progress updated deterministically; FAIL is reason-coded; if lock criteria met -> status becomes LOCKED
- side_effects: Store derived voice features; update enrollment progress state
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (onboarding_session_id + attempt_index + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### VOICE_ID_ENROLL_COMPLETE_COMMIT (COMMIT)

- name: Voice ID Enroll Complete (Commit)
- owning_domain: VoiceIdentity
- simulation_type: COMMIT
- purpose: Finalize voice profile artifact after enrollment is LOCKED
- triggers: ONB_VOICE_ENROLL_COMPLETE (example)
- required_roles: none (invitee-side)
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
onboarding_session_id: string
idempotency_key: string
```
- output_schema (minimum):
```text
onboarding_session_id: string
voice_profile_id: string
voice_enroll_status: enum (LOCKED)
```
- preconditions: voice_enroll_status is LOCKED
- postconditions: voice_profile_id exists and can be used by PH1.VOICE.ID at runtime
- side_effects: Write voice profile artifact (derived; no raw audio by default)
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (onboarding_session_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### VOICE_ID_ENROLL_DEFER_COMMIT (COMMIT)

- name: Voice ID Enroll Defer (Commit)
- owning_domain: VoiceIdentity
- simulation_type: COMMIT
- purpose: Mark voice enrollment as PENDING and request reminder scheduling (when user is busy or attempts/timeouts exhausted)
- triggers: ONB_VOICE_ENROLL_DEFER (example)
- required_roles: none (invitee-side)
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
onboarding_session_id: string
reason_code: enum (USER_BUSY | ENROLL_TIMEOUT | ENROLL_MAX_ATTEMPTS | USER_DECLINED)
idempotency_key: string
```
- output_schema (minimum):
```text
onboarding_session_id: string
voice_enroll_status: enum (PENDING)
reason_code: string
```
- preconditions: enrollment started
- postconditions: enrollment is PENDING (reason-coded); reminder may be scheduled by PH1.REM.001
- side_effects: Update enrollment state only
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (onboarding_session_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### WAKE_ENROLL_START_DRAFT (DRAFT)

- name: Wake Enroll Start (Draft)
- owning_domain: Wake
- simulation_type: DRAFT
- purpose: Start the wake-word enrollment loop (tight record -> grade -> next)
- triggers: ONB_WAKE_ENROLL_START (example)
- required_roles: none (invitee-side)
- required_approvals: none
- required_confirmations: none (DRAFT)
- input_schema (minimum):
```text
onboarding_session_id: string
device_id: string
pass_target: number (optional; bounded)
max_attempts: number (optional; bounded)
enrollment_timeout_ms: number (optional; bounded)
inter_attempt_pause_ms: number (optional; bounded)
now_ms: timestamp_ms
```
- output_schema (minimum):
```text
onboarding_session_id: string
wake_enroll_status: enum (IN_PROGRESS)
pass_target: number
max_attempts: number
enrollment_timeout_ms: number
inter_attempt_pause_ms: number
```
- deterministic defaults (if omitted):
  - pass_target=5 (allowed 3 to 8)
  - max_attempts=12 (allowed 8 to 20)
  - enrollment_timeout_ms=300000 (allowed 180000 to 600000)
  - inter_attempt_pause_ms=500 (allowed 200 to 1500)
- preconditions: onboarding_session exists; terms accepted; wake is allowed by device policy class
- postconditions: wake enrollment loop is ready for sample capture
- side_effects: Write enrollment session state only
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (onboarding_session_id + device_id)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### WAKE_ENROLL_SAMPLE_COMMIT (COMMIT)

- name: Wake Enroll Sample (Commit)
- owning_domain: Wake
- simulation_type: COMMIT
- purpose: Capture + grade one wake enrollment sample and update progress (PASS/FAIL reason-coded)
- triggers: ONB_WAKE_ENROLL_SAMPLE (example)
- required_roles: none (invitee-side)
- required_approvals: none
- required_confirmations: none (consent gate is the precondition)
- input_schema (minimum):
```text
onboarding_session_id: string
audio_sample_ref: string (bounded)
attempt_index: number
now_ms: timestamp_ms
idempotency_key: string
```
- output_schema (minimum):
```text
onboarding_session_id: string
sample_result: enum (PASS | FAIL)
reason_code: string (optional; required on FAIL)
pass_count: number
pass_target: number
wake_enroll_status: enum (IN_PROGRESS | COMPLETE | PENDING)
```
- preconditions: wake enrollment started; sample passes deterministic quality gates (else FAIL with reason_code)
- postconditions: progress updated; if pass_target reached -> COMPLETE; if max_attempts/timeout reached -> PENDING (reason-coded)
- side_effects: Store derived wake features/metrics; update per-device parameters within bounds; update enrollment progress
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (onboarding_session_id + attempt_index + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### WAKE_ENROLL_COMPLETE_COMMIT (COMMIT)

- name: Wake Enroll Complete (Commit)
- owning_domain: Wake
- simulation_type: COMMIT
- purpose: Finalize wake profile artifact after PASS target reached
- triggers: ONB_WAKE_ENROLL_COMPLETE (example)
- required_roles: none (invitee-side)
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
onboarding_session_id: string
idempotency_key: string
```
- output_schema (minimum):
```text
onboarding_session_id: string
wake_profile_id: string
wake_enroll_status: enum (COMPLETE)
```
- preconditions: wake_enroll_status is COMPLETE (pass_target reached)
- postconditions: wake_profile_id exists and can be consumed by PH1.W at runtime
- side_effects: Write wake profile artifact (derived; no raw audio by default)
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (onboarding_session_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### WAKE_ENROLL_DEFER_COMMIT (COMMIT)

- name: Wake Enroll Defer (Commit)
- owning_domain: Wake
- simulation_type: COMMIT
- purpose: Mark wake enrollment as PENDING and request reminder scheduling (when user is busy or attempts/timeouts exhausted)
- triggers: ONB_WAKE_ENROLL_DEFER (example)
- required_roles: none (invitee-side)
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
onboarding_session_id: string
reason_code: enum (USER_BUSY | ENROLL_TIMEOUT | ENROLL_MAX_ATTEMPTS | USER_DECLINED)
idempotency_key: string
```
- output_schema (minimum):
```text
onboarding_session_id: string
wake_enroll_status: enum (PENDING)
reason_code: string
```
- preconditions: wake enrollment started
- postconditions: wake enrollment is PENDING (reason-coded); reminder may be scheduled by PH1.REM.001
- side_effects: Update enrollment state only
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (onboarding_session_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### ACCESS_OVERRIDE_TEMP_GRANT_COMMIT (COMMIT)

- name: Access Override Temp Grant (Commit)
- owning_domain: Access
- simulation_type: COMMIT
- purpose: Apply an AP-approved temporary or one-shot override to a user's per-user access instance (PH2.ACCESS.002)
- triggers: ACCESS_OVERRIDE_TEMP_GRANT (example)
- required_roles: AP approver (policy)
- required_approvals: none (the actor is the approver)
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
target_user_id: string
access_engine_instance_id: string
override_scope: object (module_id + action_type; bounded)
override_mode: enum (GRANT | RESTRICT)
duration_ms: number (optional; bounded)
work_order_id: string (optional; one-shot scope)
approval_ref: string (broadcast_id or audit ref)
idempotency_key: string
```
- output_schema (minimum):
```text
override_id: string
status: APPLIED
expires_at: timestamp_ms (optional)
```
- preconditions: approver identity OK; Access policy allows this override; approval_ref is recorded
- postconditions: override is active and auditable; auto-expires when duration_ms elapses or when work_order_id completes
- side_effects: Update PH2.ACCESS.002 instance override state
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (target_user_id + override_scope + idempotency_key)
- audit_events: ACCESS_OVERRIDE_APPLIED (example)

### ACCESS_OVERRIDE_PERM_GRANT_COMMIT (COMMIT)

- name: Access Override Perm Grant (Commit)
- owning_domain: Access
- simulation_type: COMMIT
- purpose: Apply an AP-approved permanent override to a user's per-user access instance (PH2.ACCESS.002)
- triggers: ACCESS_OVERRIDE_PERM_GRANT (example)
- required_roles: AP approver (policy)
- required_approvals: none (the actor is the approver)
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
target_user_id: string
access_engine_instance_id: string
override_scope: object (module_id + action_type; bounded)
override_mode: enum (GRANT | RESTRICT)
approval_ref: string (broadcast_id or audit ref)
reason: string (bounded)
idempotency_key: string
```
- output_schema (minimum):
```text
override_id: string
status: APPLIED
```
- preconditions: approver identity OK; Access policy allows this override; approval_ref is recorded
- postconditions: permanent override is active and auditable until revoked by simulation
- side_effects: Update PH2.ACCESS.002 instance override state
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (target_user_id + override_scope + idempotency_key)
- audit_events: ACCESS_OVERRIDE_APPLIED (example)

### ACCESS_OVERRIDE_REVOKE_COMMIT (COMMIT)

- name: Access Override Revoke (Commit)
- owning_domain: Access
- simulation_type: COMMIT
- purpose: Revoke a prior override from a user's per-user access instance (PH2.ACCESS.002)
- triggers: ACCESS_OVERRIDE_REVOKE (example)
- required_roles: AP approver (policy)
- required_approvals: none (the actor is the approver)
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
target_user_id: string
access_engine_instance_id: string
override_id: string
approval_ref: string (broadcast_id or audit ref)
reason: string (bounded)
idempotency_key: string
```
- output_schema (minimum):
```text
override_id: string
status: REVOKED
```
- preconditions: override exists and is active
- postconditions: override is revoked; history remains
- side_effects: Update PH2.ACCESS.002 instance override state
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (target_user_id + override_id + idempotency_key)
- audit_events: ACCESS_OVERRIDE_REVOKED (example)

### ACCESS_AP_AUTHORING_REVIEW_CHANNEL_COMMIT (COMMIT)

- name: Access AP Authoring Review Channel Commit
- owning_domain: Access
- simulation_type: COMMIT
- purpose: Persist explicit AP authoring review channel selection (`PHONE_DESKTOP | READ_OUT_LOUD`) before AP schema lifecycle writes
- triggers: ACCESS_SCHEMA_MANAGE (authoring review channel path)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: none
- required_confirmations: required (explicit channel selection)
- input_schema (minimum):
```text
tenant_id: string (required when scope=TENANT)
scope: enum (GLOBAL | TENANT)
access_profile_id: string
schema_version_id: string
review_channel: enum (PHONE_DESKTOP | READ_OUT_LOUD)
reason_code: string
idempotency_key: string
```
- output_schema (minimum):
```text
access_profile_id: string
schema_version_id: string
review_channel: enum (PHONE_DESKTOP | READ_OUT_LOUD)
authoring_confirmation_state: REVIEW_IN_PROGRESS
```
- preconditions: caller scope is valid for requested AP scope; review channel is bounded and explicit
- postconditions: AP authoring review-channel state is persisted deterministically
- side_effects: Append AP authoring review state
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (scope_key + access_profile_id + schema_version_id + review_channel + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### ACCESS_AP_AUTHORING_RULE_ACTION_COMMIT (COMMIT)

- name: Access AP Authoring Rule Action Commit
- owning_domain: Access
- simulation_type: COMMIT
- purpose: Persist one bounded AP rule-review action (`AGREE | DISAGREE | EDIT | DELETE | DISABLE | ADD_CUSTOM_RULE`) deterministically
- triggers: ACCESS_SCHEMA_MANAGE (authoring rule action path)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: none
- required_confirmations: required (rule action commit)
- input_schema (minimum):
```text
tenant_id: string (required when scope=TENANT)
scope: enum (GLOBAL | TENANT)
access_profile_id: string
schema_version_id: string
rule_action: enum (AGREE | DISAGREE | EDIT | DELETE | DISABLE | ADD_CUSTOM_RULE)
suggested_rule_ref: string (required for AGREE | DISAGREE | EDIT | DELETE | DISABLE)
capability_id: string (required for EDIT | ADD_CUSTOM_RULE)
constraint_ref: string (optional)
escalation_policy_ref: string (optional)
reason_code: string
idempotency_key: string
```
- output_schema (minimum):
```text
access_profile_id: string
schema_version_id: string
rule_action: enum (AGREE | DISAGREE | EDIT | DELETE | DISABLE | ADD_CUSTOM_RULE)
review_action_row_id: string
```
- preconditions: review channel state exists; rule action payload passes bounded validation
- postconditions: AP rule-review action row is appended deterministically
- side_effects: Append AP authoring rule review action
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (scope_key + access_profile_id + schema_version_id + rule_action + suggested_rule_ref + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### ACCESS_AP_AUTHORING_CONFIRM_COMMIT (COMMIT)

- name: Access AP Authoring Confirm Commit
- owning_domain: Access
- simulation_type: COMMIT
- purpose: Persist final AP authoring confirmation state before activation path
- triggers: ACCESS_SCHEMA_MANAGE (authoring confirmation path)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: none (activation approvals are policy-bound in lifecycle simulations)
- required_confirmations: required (final authoring confirmation)
- input_schema (minimum):
```text
tenant_id: string (required when scope=TENANT)
scope: enum (GLOBAL | TENANT)
access_profile_id: string
schema_version_id: string
authoring_confirmation_state: enum (PENDING_ACTIVATION_CONFIRMATION | CONFIRMED_FOR_ACTIVATION | DECLINED)
reason_code: string
idempotency_key: string
```
- output_schema (minimum):
```text
access_profile_id: string
schema_version_id: string
authoring_confirmation_state: enum (CONFIRMED_FOR_ACTIVATION | DECLINED)
```
- preconditions: review channel selection exists and required rule actions are recorded
- postconditions: AP authoring confirmation state is persisted deterministically
- side_effects: Append AP authoring confirmation state
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (scope_key + access_profile_id + schema_version_id + authoring_confirmation_state + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### ACCESS_AP_SCHEMA_CREATE_DRAFT (DRAFT)

- name: Access AP Schema Create Draft
- owning_domain: Access
- simulation_type: DRAFT
- purpose: Create a schema draft row for a global or tenant access profile without activation side effects
- triggers: ACCESS_SCHEMA_MANAGE (create_draft path)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: none
- required_confirmations: required (DRAFT lifecycle authoring confirmation)
- input_schema (minimum):
```text
access_profile_id: string
schema_version_id: string
scope: enum (GLOBAL | TENANT)
tenant_id: string (required when scope=TENANT)
profile_payload_json: object (bounded)
reason_code: string
idempotency_key: string
```
- output_schema (minimum):
```text
access_profile_id: string
schema_version_id: string
lifecycle_state: DRAFT
```
- preconditions: caller scope is valid for requested AP scope; schema payload passes bounded validation
- postconditions: AP schema draft row appended; no active projection change
- side_effects: Append AP schema ledger row only
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (scope_key + access_profile_id + schema_version_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### ACCESS_AP_SCHEMA_UPDATE_COMMIT (COMMIT)

- name: Access AP Schema Update Commit
- owning_domain: Access
- simulation_type: COMMIT
- purpose: Append a deterministic update row for an AP schema draft version
- triggers: ACCESS_SCHEMA_MANAGE (update path)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: none
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
access_profile_id: string
schema_version_id: string
scope: enum (GLOBAL | TENANT)
tenant_id: string (required when scope=TENANT)
profile_payload_json: object (bounded)
reason_code: string
idempotency_key: string
```
- output_schema (minimum):
```text
access_profile_id: string
schema_version_id: string
lifecycle_state: DRAFT
```
- preconditions: target schema version exists in DRAFT
- postconditions: AP schema ledger append succeeds for update action
- side_effects: Append AP schema ledger row only
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (scope_key + access_profile_id + schema_version_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### ACCESS_AP_SCHEMA_ACTIVATE_COMMIT (COMMIT)

- name: Access AP Schema Activate Commit
- owning_domain: Access
- simulation_type: COMMIT
- purpose: Activate an AP schema version into current projection (single active version per scope/profile)
- triggers: ACCESS_SCHEMA_MANAGE (activate path)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: optional AP/board path (policy-bound)
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
access_profile_id: string
schema_version_id: string
scope: enum (GLOBAL | TENANT)
tenant_id: string (required when scope=TENANT)
reason_code: string
idempotency_key: string
```
- output_schema (minimum):
```text
access_profile_id: string
active_schema_version_id: string
lifecycle_state: ACTIVE
```
- preconditions: target schema version exists; activation policy gates are satisfied
- postconditions: current projection points to one active version for scope/profile
- side_effects: Update AP current projection
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (scope_key + access_profile_id + schema_version_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### ACCESS_AP_SCHEMA_RETIRE_COMMIT (COMMIT)

- name: Access AP Schema Retire Commit
- owning_domain: Access
- simulation_type: COMMIT
- purpose: Retire an AP schema version while preserving full history for replay
- triggers: ACCESS_SCHEMA_MANAGE (retire path)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: optional AP/board path (policy-bound)
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
access_profile_id: string
schema_version_id: string
scope: enum (GLOBAL | TENANT)
tenant_id: string (required when scope=TENANT)
reason_code: string
idempotency_key: string
```
- output_schema (minimum):
```text
access_profile_id: string
schema_version_id: string
lifecycle_state: RETIRED
```
- preconditions: target schema version exists and retirement policy allows transition
- postconditions: AP schema version marked retired; projection updated if needed
- side_effects: Update AP current projection + ledger append
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (scope_key + access_profile_id + schema_version_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### ACCESS_AP_OVERLAY_UPDATE_COMMIT (COMMIT)

- name: Access AP Overlay Update Commit
- owning_domain: Access
- simulation_type: COMMIT
- purpose: Update tenant overlay ops and active projection for one overlay version
- triggers: ACCESS_POLICY_MANAGE (overlay path)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: optional AP/board path (policy-bound)
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
tenant_id: string
overlay_id: string
overlay_version_id: string
event_action: enum (CREATE_DRAFT | UPDATE_DRAFT | ACTIVATE | RETIRE)
overlay_ops_json: object (bounded)
reason_code: string
idempotency_key: string
```
- output_schema (minimum):
```text
overlay_id: string
overlay_version_id: string
overlay_state: enum (DRAFT | ACTIVE | RETIRED)
```
- preconditions: overlay ops are bounded and tenant-scoped
- postconditions: overlay ledger append and current projection update are deterministic
- side_effects: Append overlay ledger row + update current projection
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (tenant_id + overlay_id + overlay_version_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### ACCESS_BOARD_POLICY_UPDATE_COMMIT (COMMIT)

- name: Access Board Policy Update Commit
- owning_domain: Access
- simulation_type: COMMIT
- purpose: Update board/approval policy definition and active projection deterministically
- triggers: ACCESS_POLICY_MANAGE (board policy path)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: optional board policy approval path (policy-bound)
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
tenant_id: string
board_policy_id: string
policy_version_id: string
event_action: enum (CREATE_DRAFT | UPDATE_DRAFT | ACTIVATE | RETIRE)
policy_payload_json: object (bounded)
reason_code: string
idempotency_key: string
```
- output_schema (minimum):
```text
board_policy_id: string
policy_version_id: string
policy_state: enum (DRAFT | ACTIVE | RETIRED)
```
- preconditions: policy primitive and thresholds are valid (`N_OF_M`, quorum, unanimous, mixed)
- postconditions: board policy ledger append and projection update are deterministic
- side_effects: Append board policy ledger row + update current projection
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (tenant_id + board_policy_id + policy_version_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### ACCESS_BOARD_VOTE_COMMIT (COMMIT)

- name: Access Board Vote Commit
- owning_domain: Access
- simulation_type: COMMIT
- purpose: Record one deterministic board vote row for an escalation case
- triggers: ACCESS_ESCALATION_VOTE (cast_vote path)
- required_roles: BOARD_MEMBER_ROLE_BOUND (resolved by Access policy)
- required_approvals: none
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
tenant_id: string
escalation_case_id: string
board_policy_id: string
voter_user_id: string
vote_value: enum (APPROVE | REJECT)
reason_code: string
idempotency_key: string
```
- output_schema (minimum):
```text
escalation_case_id: string
vote_row_id: string
threshold_status: enum (PENDING | SATISFIED | REJECTED)
```
- preconditions: voter is an authorized board member for tenant policy scope
- postconditions: vote row append succeeds; threshold status can be deterministically recomputed
- side_effects: Append board vote ledger row
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (tenant_id + escalation_case_id + voter_user_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### ACCESS_INSTANCE_COMPILE_COMMIT (COMMIT)

- name: Access Instance Compile Commit
- owning_domain: Access
- simulation_type: COMMIT
- purpose: Compile effective per-user access from schema chain and persist lineage refs
- triggers: ACCESS_INSTANCE_COMPILE_REFRESH (compile/refresh path)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: none (unless compile action itself is policy-gated)
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
tenant_id: string
target_user_id: string
access_profile_id: string
global_profile_version_ref: string
tenant_profile_version_ref: string (optional)
overlay_version_refs: list (optional, bounded)
position_id: string (optional)
compile_reason: string
idempotency_key: string
```
- output_schema (minimum):
```text
access_instance_id: string
compiled_global_profile_ref: string
compiled_tenant_profile_ref: string (optional)
compiled_overlay_set_ref: string (optional)
compiled_position_ref: string (optional)
```
- preconditions: all schema refs resolve and required versions are ACTIVE
- postconditions: access instance upsert succeeds with deterministic lineage refs
- side_effects: Upsert access instance lineage refs
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (tenant_id + target_user_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### CAPREQ_CREATE_DRAFT (DRAFT)

- name: Capability Request Create Draft
- owning_domain: CapabilityRequest
- simulation_type: DRAFT
- purpose: Create a deterministic capability request draft row
- triggers: CAPREQ_CREATE (example)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: none
- required_confirmations: none (DRAFT)
- input_schema (minimum):
```text
tenant_id: string
capreq_id: string
requester_user_id: string
requested_capability_id: string
justification: string (bounded)
idempotency_key: string
```
- output_schema (minimum):
```text
capreq_id: string
status: DRAFT
```
- preconditions: requester identity is verified; tenant scope is valid
- postconditions: `capreq_ledger` append succeeds; `capreq_current` reflects DRAFT
- side_effects: Append `capreq_ledger`; update `capreq_current`
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (tenant_id + capreq_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### CAPREQ_SUBMIT_FOR_APPROVAL_COMMIT (COMMIT)

- name: Capability Request Submit For Approval
- owning_domain: CapabilityRequest
- simulation_type: COMMIT
- purpose: Submit a capability request for approval
- triggers: CAPREQ_SUBMIT (example)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: none
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
tenant_id: string
capreq_id: string
requester_user_id: string
approval_route_ref: string
idempotency_key: string
```
- output_schema (minimum):
```text
capreq_id: string
status: PENDING_APPROVAL
```
- preconditions: request exists in DRAFT; requester identity is verified
- postconditions: `capreq_ledger` append succeeds; `capreq_current` reflects PENDING_APPROVAL
- side_effects: Append `capreq_ledger`; update `capreq_current`
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (tenant_id + capreq_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### CAPREQ_APPROVE_COMMIT (COMMIT)

- name: Capability Request Approve
- owning_domain: CapabilityRequest
- simulation_type: COMMIT
- purpose: Approve a capability request deterministically
- triggers: CAPREQ_APPROVE (example)
- required_roles: AP approver (policy)
- required_approvals: none (actor is approver)
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
tenant_id: string
capreq_id: string
approver_user_id: string
approval_ref: string
idempotency_key: string
```
- output_schema (minimum):
```text
capreq_id: string
status: APPROVED
```
- preconditions: request exists in PENDING_APPROVAL; approver authority is valid
- postconditions: `capreq_ledger` append succeeds; `capreq_current` reflects APPROVED
- side_effects: Append `capreq_ledger`; update `capreq_current`
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (tenant_id + capreq_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### CAPREQ_REJECT_COMMIT (COMMIT)

- name: Capability Request Reject
- owning_domain: CapabilityRequest
- simulation_type: COMMIT
- purpose: Reject a capability request deterministically
- triggers: CAPREQ_REJECT (example)
- required_roles: AP approver (policy)
- required_approvals: none (actor is approver)
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
tenant_id: string
capreq_id: string
approver_user_id: string
reject_reason: string (bounded)
idempotency_key: string
```
- output_schema (minimum):
```text
capreq_id: string
status: REJECTED
```
- preconditions: request exists in PENDING_APPROVAL
- postconditions: `capreq_ledger` append succeeds; `capreq_current` reflects REJECTED
- side_effects: Append `capreq_ledger`; update `capreq_current`
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (tenant_id + capreq_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### CAPREQ_FULFILL_COMMIT (COMMIT)

- name: Capability Request Fulfill
- owning_domain: CapabilityRequest
- simulation_type: COMMIT
- purpose: Mark an approved capability request as fulfilled after downstream execution
- triggers: CAPREQ_FULFILL (example)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
tenant_id: string
capreq_id: string
fulfillment_ref: string
idempotency_key: string
```
- output_schema (minimum):
```text
capreq_id: string
status: FULFILLED
```
- preconditions: request exists in APPROVED
- postconditions: `capreq_ledger` append succeeds; `capreq_current` reflects FULFILLED
- side_effects: Append `capreq_ledger`; update `capreq_current`
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (tenant_id + capreq_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### CAPREQ_CANCEL_REVOKE (REVOKE)

- name: Capability Request Cancel Revoke
- owning_domain: CapabilityRequest
- simulation_type: REVOKE
- purpose: Cancel an open capability request deterministically
- triggers: CAPREQ_CANCEL (example)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: none
- required_confirmations: required (REVOKE)
- input_schema (minimum):
```text
tenant_id: string
capreq_id: string
requester_user_id: string
cancel_reason: string (bounded)
idempotency_key: string
```
- output_schema (minimum):
```text
capreq_id: string
status: CANCELED
```
- preconditions: request exists and is not terminal (`FULFILLED` or `CANCELED`)
- postconditions: `capreq_ledger` append succeeds; `capreq_current` reflects CANCELED
- side_effects: Append `capreq_ledger`; update `capreq_current`
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (tenant_id + capreq_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### REMINDER_SCHEDULE_COMMIT (COMMIT)

- name: Reminder Schedule (Commit)
- owning_domain: Reminder
- simulation_type: COMMIT
- purpose: Schedule a reminder deterministically (timezone/DST/recurrence bounded)
- triggers: REMINDER_CREATE (example)
- required_roles: self (owner) or policy
- required_approvals: none
- required_confirmations: required when time is ambiguous; otherwise none
- input_schema (minimum):
```text
reminder_request_text: string (bounded)
user_id: string
device_id: string
user_timezone: string
local_time_mode: enum (FIXED_TIMEZONE | LOCAL_TIME)
priority_level: enum (LOW | NORMAL | HIGH | CRITICAL)
reminder_type: enum (TASK | MEETING | TIMER | MEDICAL | CUSTOM | BCAST_MHP_FOLLOWUP)
desired_time: datetime or duration
recurrence_rule: string (optional; bounded)
channel_preferences: list (ordered)
delivery_channel: enum (voice | push | text | email)
idempotency_key: string
now_ms: timestamp_ms
```
- output_schema (minimum):
```text
reminder_id: string
scheduled_time: timestamp_ms
occurrence_id: string
state: SCHEDULED
```
- preconditions: identity OK; time resolved/confirmed; policy snapshot available
- postconditions: reminder is scheduled; recurrence expansion bounded (max 365 occurrences ahead)
- side_effects: Write reminder record + occurrence schedule
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (user_id + scheduled_time + reminder_type + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### REMINDER_UPDATE_COMMIT (COMMIT)

- name: Reminder Update (Commit)
- owning_domain: Reminder
- simulation_type: COMMIT
- purpose: Update reminder fields deterministically
- triggers: REMINDER_UPDATE (example)
- required_roles: self (owner) or policy
- required_approvals: none
- required_confirmations: required when new time is ambiguous; otherwise none
- input_schema (minimum):
```text
reminder_id: string
user_id: string
updated_fields: object (bounded)
idempotency_key: string
now_ms: timestamp_ms
```
- output_schema (minimum):
```text
reminder_id: string
updated_fields: object (bounded)
state: SCHEDULED
```
- preconditions: reminder exists and is not COMPLETED/CANCELED
- postconditions: reminder updated deterministically; next occurrence schedule adjusted if needed
- side_effects: State write only
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (reminder_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### REMINDER_CANCEL_COMMIT (COMMIT)

- name: Reminder Cancel (Commit)
- owning_domain: Reminder
- simulation_type: COMMIT
- purpose: Cancel a reminder deterministically
- triggers: REMINDER_CANCEL (example)
- required_roles: self (owner) or policy
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
reminder_id: string
user_id: string
cancel_reason: string (bounded; optional)
idempotency_key: string
now_ms: timestamp_ms
```
- output_schema (minimum):
```text
reminder_id: string
state: CANCELED
```
- preconditions: reminder exists
- postconditions: reminder canceled; pending follow-ups canceled
- side_effects: State write only
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (reminder_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### REMINDER_SNOOZE_COMMIT (COMMIT)

- name: Reminder Snooze (Commit)
- owning_domain: Reminder
- simulation_type: COMMIT
- purpose: Snooze a reminder by a bounded duration
- triggers: REMINDER_SNOOZE (example)
- required_roles: self (owner) or policy
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
reminder_id: string
occurrence_id: string
user_id: string
snooze_duration_ms: int
idempotency_key: string
now_ms: timestamp_ms
```
- output_schema (minimum):
```text
reminder_id: string
occurrence_id: string
snooze_until_ms: timestamp_ms
state: SNOOZED
```
- preconditions: reminder occurrence is due or followup pending; snooze within bounds
- postconditions: next delivery time updated deterministically
- side_effects: State write only
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (reminder_id + occurrence_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### REMINDER_DELIVER_PRE_COMMIT (COMMIT)

- name: Reminder Deliver Pre-Alert (Commit)
- owning_domain: Reminder
- simulation_type: COMMIT
- purpose: Deliver a pre-reminder alert (if configured)
- triggers: REMINDER_PRE_ALERT (example)
- required_roles: self (owner) or policy
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
reminder_id: string
occurrence_id: string
user_id: string
delivery_channel: enum (voice | push | text | email)
delivery_attempt_id: string
now_ms: timestamp_ms
```
- output_schema (minimum):
```text
reminder_id: string
occurrence_id: string
delivery_status: enum (DELIVERED | DEFERRED_QUIET_HOURS | FAIL)
delivery_proof_ref: string (optional)
```
- preconditions: occurrence scheduled; pre-alert configured; quiet-hours policy evaluated deterministically
- postconditions: pre-alert delivered or deferred with reason code; idempotent proof recorded
- side_effects: Deliver via channel adapter; write proof
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (reminder_id + occurrence_id + delivery_attempt_id)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### REMINDER_DELIVER_DUE_COMMIT (COMMIT)

- name: Reminder Deliver Due (Commit)
- owning_domain: Reminder
- simulation_type: COMMIT
- purpose: Deliver the due reminder at scheduled_time (respect quiet hours policy)
- triggers: REMINDER_DUE (example)
- required_roles: self (owner) or policy
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
reminder_id: string
occurrence_id: string
user_id: string
delivery_channel: enum (voice | push | text | email)
delivery_attempt_id: string
offline_state: enum (online | offline)
now_ms: timestamp_ms
```
- output_schema (minimum):
```text
reminder_id: string
occurrence_id: string
delivery_status: enum (DELIVERED | DEFERRED_QUIET_HOURS | RETRY_SCHEDULED | FAIL)
delivery_proof_ref: string (optional)
```
- preconditions: scheduled_time reached (or overdue within policy); quiet-hours policy evaluated deterministically
- postconditions: due reminder delivered, deferred, retried, or failed with deterministic reason code
- side_effects: Deliver via channel adapter; write proof; schedule retry/follow-up if needed
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (reminder_id + occurrence_id + delivery_attempt_id)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### REMINDER_FOLLOWUP_SCHEDULE_COMMIT (COMMIT)

- name: Reminder Follow-up Schedule (Commit)
- owning_domain: Reminder
- simulation_type: COMMIT
- purpose: Schedule a follow-up when no acknowledgment is received
- triggers: REMINDER_FOLLOWUP (example)
- required_roles: self (owner) or policy
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
reminder_id: string
occurrence_id: string
user_id: string
followup_delay_ms: int
idempotency_key: string
now_ms: timestamp_ms
```
- output_schema (minimum):
```text
reminder_id: string
occurrence_id: string
followup_time_ms: timestamp_ms
state: FOLLOWUP_PENDING
```
- preconditions: due was sent; no acknowledgment; follow-up attempts remaining
- postconditions: follow-up time scheduled deterministically
- side_effects: State write only
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (reminder_id + occurrence_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### REMINDER_ESCALATE_COMMIT (COMMIT)

- name: Reminder Escalate Channel (Commit)
- owning_domain: Reminder
- simulation_type: COMMIT
- purpose: Escalate reminder delivery to the next channel deterministically
- triggers: REMINDER_ESCALATE (example)
- required_roles: self (owner) or policy
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
reminder_id: string
occurrence_id: string
user_id: string
from_channel: enum (voice | push | text | email)
to_channel: enum (voice | push | text | email)
delivery_attempt_id: string
now_ms: timestamp_ms
```
- output_schema (minimum):
```text
reminder_id: string
occurrence_id: string
delivery_status: enum (DELIVERED | DEFERRED_QUIET_HOURS | FAIL)
escalation_level: int
delivery_proof_ref: string (optional)
```
- preconditions: follow-up attempts/ignore policy triggers escalation; channel preferences available
- postconditions: escalated delivery attempt recorded deterministically
- side_effects: Deliver via escalated channel adapter; write proof
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (reminder_id + occurrence_id + delivery_attempt_id)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### REMINDER_DELIVERY_RETRY_SCHEDULE_COMMIT (COMMIT)

- name: Reminder Delivery Retry Schedule (Commit)
- owning_domain: Reminder
- simulation_type: COMMIT
- purpose: Schedule a bounded delivery retry after a delivery failure
- triggers: REMINDER_RETRY (example)
- required_roles: self (owner) or policy
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
reminder_id: string
occurrence_id: string
user_id: string
retry_time_ms: timestamp_ms
idempotency_key: string
now_ms: timestamp_ms
```
- output_schema (minimum):
```text
reminder_id: string
occurrence_id: string
retry_time_ms: timestamp_ms
```
- preconditions: delivery failed with retryable reason; retry count within bounds
- postconditions: retry scheduled deterministically
- side_effects: State write only
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (reminder_id + occurrence_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### REMINDER_MARK_COMPLETED_COMMIT (COMMIT)

- name: Reminder Mark Completed (Commit)
- owning_domain: Reminder
- simulation_type: COMMIT
- purpose: Mark reminder occurrence as completed on user acknowledgment
- triggers: REMINDER_ACK_DONE (example)
- required_roles: self (owner) or policy
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
reminder_id: string
occurrence_id: string
user_id: string
ack_source: enum (voice | text | ui)
idempotency_key: string
now_ms: timestamp_ms
```
- output_schema (minimum):
```text
reminder_id: string
occurrence_id: string
state: COMPLETED
completion_time_ms: timestamp_ms
```
- preconditions: reminder exists
- postconditions: occurrence completed; pending follow-ups canceled deterministically
- side_effects: State write only
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (reminder_id + occurrence_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### REMINDER_MARK_FAILED_COMMIT (COMMIT)

- name: Reminder Mark Failed (Commit)
- owning_domain: Reminder
- simulation_type: COMMIT
- purpose: Mark reminder occurrence as failed after retries/max attempts exhausted
- triggers: REMINDER_DELIVERY_FAIL_FINAL (example)
- required_roles: self (owner) or policy
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
reminder_id: string
occurrence_id: string
user_id: string
failure_reason: string (bounded)
idempotency_key: string
now_ms: timestamp_ms
```
- output_schema (minimum):
```text
reminder_id: string
occurrence_id: string
state: FAILED
```
- preconditions: retries and max attempts exhausted deterministically
- postconditions: occurrence failed; no further retries/follow-ups
- side_effects: State write only
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (reminder_id + occurrence_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### EMO_SIM_001 (COMMIT)

- name: EMO Classify Profile
- owning_domain: Emotional
- simulation_type: COMMIT
- purpose: Classify and (when possible) lock the user's personality_type (PASSIVE/DOMINEERING/UNDETERMINED) for tone continuity (tone-only)
- triggers: EMO_CLASSIFY_PROFILE (example)
- required_roles: self (system) (identity verified)
- required_approvals: none
- required_confirmations: none (consent gate is the precondition)
- input_schema (minimum):
```text
user_id: string
session_id: string
consent_asserted: bool
identity_verified: bool
signals: object (assertive_score/distress_score/anger_score/warmth_signal)
timestamp_ms: timestamp_ms
idempotency_key: string
```
- output_schema (minimum):
```text
user_id: string
personality_type: enum (PASSIVE | DOMINEERING | UNDETERMINED)
personality_lock_status: enum (LOCKED | REEVAL_DUE | REEVAL_CHANGED | REEVAL_CONFIRMED)
voice_style_profile: object (pace_bucket, energy_bucket, warmth_bucket)
reason_code: string
```
- preconditions: consent_asserted=true; identity_verified=true
- postconditions: emotional profile updated deterministically (tone-only)
- side_effects: Update emotional profile record
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (user_id + session_id + idempotency_key)
- audit_events: EMO_PROFILE_UPDATED (example)

### EMO_SIM_002 (COMMIT)

- name: EMO Re-evaluate Profile
- owning_domain: Emotional
- simulation_type: COMMIT
- purpose: Re-evaluate and update the user's personality profile at deterministic gates (10 sessions or 21 days)
- triggers: EMO_REEVALUATE_PROFILE (example)
- required_roles: self (system) (identity verified)
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
user_id: string
session_id: string
consent_asserted: bool
identity_verified: bool
signals_window_ref: string (bounded reference to recent signals)
timestamp_ms: timestamp_ms
idempotency_key: string
```
- output_schema (minimum):
```text
user_id: string
personality_type: enum (PASSIVE | DOMINEERING | UNDETERMINED)
personality_lock_status: enum (LOCKED | REEVAL_DUE | REEVAL_CHANGED | REEVAL_CONFIRMED)
reason_code: string
```
- preconditions: re-eval gate is due; consent_asserted=true; identity_verified=true
- postconditions: emotional profile updated or confirmed deterministically
- side_effects: Update emotional profile record
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (user_id + idempotency_key)
- audit_events: EMO_PROFILE_REEVALUATED (example)

### EMO_SIM_003 (COMMIT)

- name: EMO Apply Privacy Command
- owning_domain: Emotional
- simulation_type: COMMIT
- purpose: Apply emotional privacy commands (forget/do-not-remember/recall-only/archive) deterministically
- triggers: EMO_PRIVACY_COMMAND (example)
- required_roles: self (user) (identity verified)
- required_approvals: none
- required_confirmations: required for destructive commands (policy)
- input_schema (minimum):
```text
user_id: string
session_id: string
privacy_command: enum (FORGET_THIS_KEY | FORGET_ALL | DO_NOT_REMEMBER | RECALL_ONLY | KEEP_ACTIVE | ARCHIVE)
target_key: string (optional; bounded)
timestamp_ms: timestamp_ms
idempotency_key: string
```
- output_schema (minimum):
```text
user_id: string
privacy_state: enum (KEEP_ACTIVE | DO_NOT_REMEMBER | RECALL_ONLY | ARCHIVE)
reason_code: string
```
- preconditions: identity_verified=true
- postconditions: emotional privacy policy state updated deterministically
- side_effects: Update emotional privacy policy state
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (user_id + privacy_command + idempotency_key)
- audit_events: EMO_PRIVACY_APPLIED (example)

### EMO_SIM_004 (DRAFT)

- name: EMO Emit Tone Guidance (Draft)
- owning_domain: Emotional
- simulation_type: DRAFT
- purpose: Emit per-turn tone guidance from the current emotional profile and signals (no profile mutation)
- triggers: EMO_TONE_GUIDANCE (example)
- required_roles: none (draft)
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
user_id: string (optional; if unknown -> neutral)
profile_snapshot_ref: string (optional)
signals: object (bounded)
timestamp_ms: timestamp_ms
```
- output_schema (minimum):
```text
tone_guidance: object (voice_style_tags, pacing_guidance, directness, empathy_level)
reason_code: string
```
- preconditions: none
- postconditions: none
- side_effects: None (output only)
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: N/A
- audit_events: optional (audit-only)

### EMO_SIM_005 (COMMIT)

- name: EMO Snapshot Capture (Onboarding)
- owning_domain: Emotional
- simulation_type: COMMIT
- purpose: Capture an onboarding snapshot for emotional profile initialization (non-blocking)
- triggers: ONB_EMO_SNAPSHOT (example)
- required_roles: self (system) (identity verified)
- required_approvals: none
- required_confirmations: none (consent gate is the precondition)
- input_schema (minimum):
```text
user_id: string
onboarding_session_id: string
consent_asserted: bool
identity_verified: bool
signals: object (bounded)
timestamp_ms: timestamp_ms
idempotency_key: string
```
- output_schema (minimum):
```text
user_id: string
snapshot_ref: string
snapshot_status: enum (COMPLETE | DEFER)
reason_code: string
```
- preconditions: onboarding in progress; consent_asserted=true; identity_verified=true
- postconditions: snapshot stored; onboarding continues even if DEFER
- side_effects: Write snapshot record
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (onboarding_session_id + idempotency_key)
- audit_events: EMO_SNAPSHOT_CAPTURED (example)

### EMO_SIM_006 (COMMIT)

- name: EMO Emit Audit Event
- owning_domain: Emotional
- simulation_type: COMMIT
- purpose: Emit an audit-grade emotional event with reason codes (explicit record)
- triggers: EMO_AUDIT_EVENT (example)
- required_roles: self (system)
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
user_id: string
session_id: string
event_type: string (bounded)
reason_codes: list (bounded)
timestamp_ms: timestamp_ms
idempotency_key: string
```
- output_schema (minimum):
```text
event_id: string
status: RECORDED
```
- preconditions: none
- postconditions: audit event exists
- side_effects: Append audit event
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (user_id + session_id + idempotency_key)
- audit_events: EMO_AUDIT_RECORDED (example)

### BCAST-SIM-001 (DRAFT)

- name: Create Broadcast Draft
- owning_domain: Broadcast
- simulation_type: DRAFT
- purpose: Create a broadcast draft envelope (no delivery yet)
- triggers: BCAST_CREATE (example)
- required_roles: sender must be authenticated; authority checked separately
- required_approvals: none (draft)
- required_confirmations: none (draft)
- input_schema (minimum):
```text
sender_id: string
origin_context: string (bounded)
classification: enum (Simple | Priority | Private | Confidential | Emergency)
audience_spec: object (bounded selectors + overrides)
delivery_policy: object (bounded; retries/quiet-hours limits)
content_payload: object (structured; truth)
content_language: string (BCP-47)
required_ack: enum (None | Read | Confirm | Action-Confirm)
expiry_ms: timestamp_ms
idempotency_key: string
now_ms: timestamp_ms
```
- output_schema (minimum):
```text
broadcast_id: string
envelope_hash: string
status: DRAFT_CREATED
```
- preconditions: sender identity OK; inputs schema-valid
- postconditions: broadcast draft exists; no delivery performed
- side_effects: Write broadcast draft record only
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (sender_id + envelope_hash + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### BCAST-SIM-002 (DRAFT)

- name: Validate Sender Authority (Broadcast)
- owning_domain: Broadcast
- simulation_type: DRAFT
- purpose: Validate sender authority for classification/audience/channels under policy
- triggers: BCAST_VALIDATE (example)
- required_roles: Access-owned (policy-dependent)
- required_approvals: optional AP approval (policy-dependent)
- required_confirmations: none (validation)
- input_schema (minimum):
```text
broadcast_id: string
sender_id: string
classification: enum
audience_spec: object
requested_channels: list (ordered)
policy_snapshot_ref: string
now_ms: timestamp_ms
```
- output_schema (minimum):
```text
broadcast_id: string
authority_result: enum (ALLOW | DENY | ESCALATE)
reason_code: string
```
- preconditions: broadcast draft exists; policy snapshot available
- postconditions: validation recorded deterministically
- side_effects: Write validation result only
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (broadcast_id + policy_snapshot_ref)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### BCAST-SIM-003 (DRAFT)

- name: Privacy Handshake Decision
- owning_domain: Broadcast
- simulation_type: DRAFT
- purpose: Resolve recipient privacy choice (OutLoud/DeviceOnly/Mixed) before content
- triggers: BCAST_PRIVACY (example)
- required_roles: none (recipient-side choice), but classification may restrict options
- required_approvals: none
- required_confirmations: required if privacy_choice is Unknown and classification demands it
- input_schema (minimum):
```text
broadcast_id: string
recipient_id: string
classification: enum
recipient_privacy_pref: enum (OutLoud | DeviceOnly | Mixed | Unknown)
privacy_choice: enum (OutLoud | DeviceOnly | Mixed | Unknown)
now_ms: timestamp_ms
```
- output_schema (minimum):
```text
broadcast_id: string
recipient_id: string
privacy_choice: enum (OutLoud | DeviceOnly | Mixed)
```
- preconditions: broadcast exists; recipient in audience
- postconditions: privacy choice recorded for this recipient
- side_effects: State write only (recipient privacy_choice)
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (broadcast_id + recipient_id)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### BCAST-SIM-004 (COMMIT)

- name: Deliver To Recipient
- owning_domain: Broadcast
- simulation_type: COMMIT
- purpose: Deliver to a recipient (handshake -> privacy -> content -> ack requirements)
- triggers: BCAST_DELIVER (example)
- required_roles: sender authority must be ALLOW; recipient delivery must respect policy
- required_approvals: none (delivery), unless classification requires step-up gates (Access-owned)
- required_confirmations: required when classification/policy requires explicit confirmation
- input_schema (minimum):
```text
broadcast_id: string
recipient_id: string
delivery_channel: enum (voice | push | text | email)
rendered_language: string (BCP-47)
privacy_choice: enum (OutLoud | DeviceOnly | Mixed)
delivery_attempt_id: string
now_ms: timestamp_ms
```
- output_schema (minimum):
```text
broadcast_id: string
recipient_id: string
recipient_status: enum (Requested-Availability | Delivered | Deferred | Rejected | Escalated | Expired)
delivery_proof_ref: string (optional)
reason_code: string
```
- preconditions: broadcast is active; recipient in audience; expiry not reached; quiet-hours policy evaluated deterministically
- postconditions: recipient status updated; proof recorded if delivered
- side_effects: Deliver via channel adapter + write proof (idempotent)
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (broadcast_id + recipient_id + delivery_attempt_id)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### BCAST-SIM-005 (COMMIT)

- name: Defer And Schedule Retry
- owning_domain: Broadcast
- simulation_type: COMMIT
- purpose: Record defer and schedule retry/follow-up (handoff to Reminder Engine when needed)
- triggers: BCAST_DEFER (example)
- required_roles: none (recipient-side defer)
- required_approvals: none
- required_confirmations: required if defer time is ambiguous
- input_schema (minimum):
```text
broadcast_id: string
recipient_id: string
defer_time_ms: timestamp_ms (optional)
defer_duration_ms: int (optional)
idempotency_key: string
now_ms: timestamp_ms
```
- output_schema (minimum):
```text
broadcast_id: string
recipient_id: string
next_attempt_at_ms: timestamp_ms
handoff_to_reminder: bool
reason_code: string
```
- preconditions: broadcast active; recipient not acknowledged; defer within bounds
- postconditions: recipient status becomes Deferred; next_attempt_at set deterministically; reminder handoff request may be emitted
- side_effects: State write only + (optional) emit reminder request for PH1.REM
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (broadcast_id + recipient_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### BCAST-SIM-006 (COMMIT)

- name: Acknowledge Received
- owning_domain: Broadcast
- simulation_type: COMMIT
- purpose: Record recipient acknowledgment (Read/Confirm/Action-Confirm)
- triggers: BCAST_ACK (example)
- required_roles: none (recipient-side ack)
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
broadcast_id: string
recipient_id: string
ack_type: enum (Read | Confirm | Action-Confirm)
ack_source: enum (voice | text | ui)
idempotency_key: string
now_ms: timestamp_ms
```
- output_schema (minimum):
```text
broadcast_id: string
recipient_id: string
recipient_status: Acknowledged
```
- preconditions: broadcast delivered; ack satisfies required_ack policy
- postconditions: recipient acknowledged; follow-ups canceled deterministically
- side_effects: State write only
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (broadcast_id + recipient_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### BCAST-SIM-007 (COMMIT)

- name: Escalate To Sender
- owning_domain: Broadcast
- simulation_type: COMMIT
- purpose: Escalate to sender when blocked/unreachable/verification failed
- triggers: BCAST_ESCALATE (example)
- required_roles: sender must be reachable; escalation policy must allow it
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
broadcast_id: string
sender_id: string
recipient_id: string (optional)
reason_code: string
delivery_attempt_id: string
now_ms: timestamp_ms
```
- output_schema (minimum):
```text
broadcast_id: string
escalation_status: enum (SENT | FAIL)
delivery_proof_ref: string (optional)
```
- preconditions: escalation rules triggered; broadcast not expired
- postconditions: escalation attempt recorded deterministically
- side_effects: Deliver escalation message + proof (idempotent)
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (broadcast_id + delivery_attempt_id)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### BCAST-SIM-008 (COMMIT)

- name: Expire Broadcast
- owning_domain: Broadcast
- simulation_type: COMMIT
- purpose: Expire a broadcast (hard stop) and mark remaining recipients as expired
- triggers: BCAST_EXPIRE (example)
- required_roles: policy (system)
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
broadcast_id: string
now_ms: timestamp_ms
```
- output_schema (minimum):
```text
broadcast_id: string
status: EXPIRED
```
- preconditions: expiry reached or sender cancels via policy
- postconditions: broadcast expired; remaining pending recipients are marked expired
- side_effects: State write only; optional sender notification
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (broadcast_id)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### BCAST_CREATE_DRAFT (DRAFT)

- name: Broadcast Create Draft (Canonical)
- owning_domain: Broadcast
- simulation_type: DRAFT
- purpose: Create deterministic broadcast envelope draft before any delivery commit path
- triggers: BCAST_DRAFT_CREATE (process step)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: none
- required_confirmations: none (draft)
- input_schema (minimum):
```text
tenant_id: string
sender_user_id: string
classification: enum (SIMPLE | PRIORITY | PRIVATE | CONFIDENTIAL | EMERGENCY)
audience_spec: object (bounded)
content_payload_ref: string
expiry_at: timestamp_ms (optional)
idempotency_key: string
```
- output_schema (minimum):
```text
broadcast_id: string
status: DRAFT_CREATED
envelope_hash: string
```
- preconditions: sender identity verified; tenant scope valid; requested classification allowed by policy
- postconditions: envelope draft exists; no external delivery call performed
- side_effects: append lifecycle draft event + update envelope current projection
- reads_tables[]: [`comms.broadcast_envelopes_current`]
- writes_tables[]: [`comms.broadcast_envelopes_ledger`, `comms.broadcast_envelopes_current`]
- idempotency_key_rule: idempotent on (tenant_id + sender_user_id + envelope_hash + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### BCAST_DELIVER_COMMIT (COMMIT)

- name: Broadcast Deliver Commit (Canonical)
- owning_domain: Broadcast
- simulation_type: COMMIT
- purpose: Commit one recipient delivery request for an approved broadcast envelope
- triggers: BCAST_DELIVER_COMMIT_STEP (process step)
- required_roles: POLICY_ROLE_BOUND + ACCESS_ALLOWED
- required_approvals: policy-dependent (already resolved before commit)
- required_confirmations: required for sender-confirm flows
- input_schema (minimum):
```text
tenant_id: string
broadcast_id: string
recipient_id: string
delivery_channel: enum (VOICE | PUSH | TEXT | EMAIL)
delivery_payload_ref: string
simulation_context: object
idempotency_key: string
```
- output_schema (minimum):
```text
broadcast_id: string
recipient_id: string
recipient_status: enum (REQUESTED_AVAILABILITY | DELIVERED | DEFERRED | REJECTED | ESCALATED)
delivery_request_ref: string
```
- preconditions: access decision is ALLOW; simulation context present; envelope not expired/canceled
- postconditions: recipient transition recorded; delivery request is eligible for PH1.DELIVERY send commit
- side_effects: append recipient delivery attempt + recipient state transition
- reads_tables[]: [`comms.broadcast_envelopes_current`, `comms.broadcast_recipients_current`]
- writes_tables[]: [`comms.broadcast_delivery_attempts_ledger`, `comms.broadcast_recipients_current`]
- idempotency_key_rule: idempotent on (tenant_id + broadcast_id + recipient_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### BCAST_DEFER_COMMIT (COMMIT)

- name: Broadcast Defer Commit (Canonical)
- owning_domain: Broadcast
- simulation_type: COMMIT
- purpose: Commit defer choice and deterministic retry schedule for a broadcast recipient
- triggers: BCAST_DEFER_COMMIT_STEP (process step)
- required_roles: recipient or policy-authorized actor
- required_approvals: none
- required_confirmations: required if defer target time is ambiguous
- input_schema (minimum):
```text
tenant_id: string
broadcast_id: string
recipient_id: string
defer_until: timestamp_ms (optional)
defer_duration_ms: int (optional)
idempotency_key: string
```
- output_schema (minimum):
```text
broadcast_id: string
recipient_id: string
recipient_status: DEFERRED
next_retry_at: timestamp_ms
```
- preconditions: recipient is in pending or delivered-without-ack state; defer policy bounds pass
- postconditions: deterministic retry schedule persisted
- side_effects: append defer lifecycle event + update recipient retry schedule
- reads_tables[]: [`comms.broadcast_recipients_current`]
- writes_tables[]: [`comms.broadcast_delivery_attempts_ledger`, `comms.broadcast_recipients_current`]
- idempotency_key_rule: idempotent on (tenant_id + broadcast_id + recipient_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### BCAST_REMINDER_FIRED_COMMIT (COMMIT)

- name: Broadcast Reminder Fired Commit (Canonical)
- owning_domain: Broadcast
- simulation_type: COMMIT
- purpose: Resume BCAST.MHP lifecycle after REM timer fires (`REMINDER_SET -> REMINDER_FIRED`)
- triggers: BCAST_MHP_REMINDER_FIRED_STEP (process step)
- required_roles: policy-authorized system actor
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
tenant_id: string
broadcast_id: string
recipient_id: string
reminder_ref: string
idempotency_key: string
```
- output_schema (minimum):
```text
broadcast_id: string
recipient_id: string
recipient_status: REMINDER_FIRED
reminder_ref: string
```
- preconditions: recipient is in REMINDER_SET state
- postconditions: recipient state becomes REMINDER_FIRED; next BCAST follow-up step is eligible
- side_effects: append reminder-fired lifecycle event + recipient state transition
- reads_tables[]: [`comms.broadcast_recipients_current`]
- writes_tables[]: [`comms.broadcast_delivery_attempts_ledger`, `comms.broadcast_recipients_current`]
- idempotency_key_rule: idempotent on (tenant_id + broadcast_id + recipient_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### BCAST_ACK_COMMIT (COMMIT)

- name: Broadcast Ack Commit (Canonical)
- owning_domain: Broadcast
- simulation_type: COMMIT
- purpose: Commit recipient acknowledgment against required ack mode
- triggers: BCAST_ACK_COMMIT_STEP (process step)
- required_roles: recipient or policy-authorized actor
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
tenant_id: string
broadcast_id: string
recipient_id: string
ack_type: enum (READ | CONFIRM | ACTION_CONFIRM)
idempotency_key: string
```
- output_schema (minimum):
```text
broadcast_id: string
recipient_id: string
recipient_status: ACKNOWLEDGED
ack_record_id: string
```
- preconditions: recipient received broadcast; ack_type satisfies envelope required_ack
- postconditions: ack state recorded and pending retries canceled
- side_effects: append ack event + recipient state transition
- reads_tables[]: [`comms.broadcast_envelopes_current`, `comms.broadcast_recipients_current`]
- writes_tables[]: [`comms.broadcast_ack_ledger`, `comms.broadcast_recipients_current`]
- idempotency_key_rule: idempotent on (tenant_id + broadcast_id + recipient_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### BCAST_ESCALATE_COMMIT (COMMIT)

- name: Broadcast Escalate Commit (Canonical)
- owning_domain: Broadcast
- simulation_type: COMMIT
- purpose: Commit escalation lifecycle update to sender for blocked or unreachable recipients
- triggers: BCAST_ESCALATE_COMMIT_STEP (process step)
- required_roles: policy-authorized system actor
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
tenant_id: string
broadcast_id: string
recipient_id: string (optional)
escalation_reason_code: string
idempotency_key: string
```
- output_schema (minimum):
```text
broadcast_id: string
escalation_status: enum (RECORDED | SENT)
recipient_status: ESCALATED
```
- preconditions: recipient has retry exhausted or policy-blocked path
- postconditions: escalation recorded and recipient state updated
- side_effects: append escalation lifecycle event
- reads_tables[]: [`comms.broadcast_recipients_current`]
- writes_tables[]: [`comms.broadcast_delivery_attempts_ledger`, `comms.broadcast_recipients_current`]
- idempotency_key_rule: idempotent on (tenant_id + broadcast_id + escalation_reason_code + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### BCAST_EXPIRE_COMMIT (COMMIT)

- name: Broadcast Expire Commit (Canonical)
- owning_domain: Broadcast
- simulation_type: COMMIT
- purpose: Commit broadcast expiry and close unresolved recipient states
- triggers: BCAST_EXPIRE_COMMIT_STEP (process step)
- required_roles: policy-authorized system actor
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
tenant_id: string
broadcast_id: string
expired_at: timestamp_ms
idempotency_key: string
```
- output_schema (minimum):
```text
broadcast_id: string
status: EXPIRED
expired_recipients_count: int
```
- preconditions: envelope exists and not terminal; expiry policy reached
- postconditions: envelope closed and unresolved recipient states marked expired
- side_effects: append expire lifecycle event + envelope close projection
- reads_tables[]: [`comms.broadcast_envelopes_current`, `comms.broadcast_recipients_current`]
- writes_tables[]: [`comms.broadcast_envelopes_ledger`, `comms.broadcast_envelopes_current`, `comms.broadcast_recipients_current`]
- idempotency_key_rule: idempotent on (tenant_id + broadcast_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### BCAST_CANCEL_COMMIT (COMMIT)

- name: Broadcast Cancel Commit (Canonical)
- owning_domain: Broadcast
- simulation_type: COMMIT
- purpose: Commit sender/policy cancellation of a broadcast before expiry
- triggers: BCAST_CANCEL_COMMIT_STEP (process step)
- required_roles: sender authority or policy-authorized actor
- required_approvals: policy-dependent
- required_confirmations: required for sender-triggered cancel
- input_schema (minimum):
```text
tenant_id: string
broadcast_id: string
cancel_reason_code: string
idempotency_key: string
```
- output_schema (minimum):
```text
broadcast_id: string
status: CANCELED
```
- preconditions: envelope exists and not terminal
- postconditions: envelope canceled; unresolved recipients closed with cancel reason
- side_effects: append cancel lifecycle event + envelope close projection
- reads_tables[]: [`comms.broadcast_envelopes_current`, `comms.broadcast_recipients_current`]
- writes_tables[]: [`comms.broadcast_envelopes_ledger`, `comms.broadcast_envelopes_current`, `comms.broadcast_recipients_current`]
- idempotency_key_rule: idempotent on (tenant_id + broadcast_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### DELIVERY_SEND_COMMIT (COMMIT)

- name: Delivery Send Commit (Canonical)
- owning_domain: Delivery
- simulation_type: COMMIT
- purpose: Commit provider send request and persist delivery proof reference
- triggers: DELIVERY_SEND_COMMIT_STEP (process step)
- required_roles: POLICY_ROLE_BOUND + ACCESS_ALLOWED
- required_approvals: none (already resolved pre-commit)
- required_confirmations: none
- input_schema (minimum):
```text
tenant_id: string
message_id: string
recipient: string
channel: enum (SMS | EMAIL | WHATSAPP | WECHAT | APP_PUSH)
payload_hash: string
provider: string
simulation_context: object
idempotency_key: string
```
- output_schema (minimum):
```text
delivery_attempt_id: string
provider_message_ref: string (optional)
delivery_status: enum (QUEUED | SENT | FAILED)
```
- preconditions: simulation_context present; provider routing allowed by policy; secrets resolved via handles
- postconditions: provider attempt persisted with deterministic status mapping
- side_effects: append provider send attempt + update current attempt projection
- reads_tables[]: [`comms.delivery_attempts_current`, `comms.delivery_provider_health`]
- writes_tables[]: [`comms.delivery_attempts_ledger`, `comms.delivery_attempts_current`]
- idempotency_key_rule: idempotent on (tenant_id + message_id + recipient + channel + payload_hash + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### DELIVERY_CANCEL_COMMIT (COMMIT)

- name: Delivery Cancel Commit (Canonical)
- owning_domain: Delivery
- simulation_type: COMMIT
- purpose: Commit provider cancel request for an in-flight delivery attempt
- triggers: DELIVERY_CANCEL_COMMIT_STEP (process step)
- required_roles: policy-authorized actor
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
tenant_id: string
delivery_attempt_id: string
provider: string
simulation_context: object
idempotency_key: string
```
- output_schema (minimum):
```text
delivery_attempt_id: string
cancel_status: enum (CANCELED | NOT_SUPPORTED | FAILED)
```
- preconditions: simulation_context present; attempt exists and is cancel-eligible
- postconditions: cancel result recorded deterministically and projected
- side_effects: append provider cancel attempt + update current attempt projection
- reads_tables[]: [`comms.delivery_attempts_current`]
- writes_tables[]: [`comms.delivery_attempts_ledger`, `comms.delivery_attempts_current`]
- idempotency_key_rule: idempotent on (tenant_id + delivery_attempt_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### SMS_SETUP_SIM (COMMIT)

- name: SMS App Setup Commit
- owning_domain: OnboardingSms
- simulation_type: COMMIT
- purpose: Commit one user SMS app setup state transition for send/receive readiness
- triggers: MESSAGE_COMPOSE_AND_SEND and onboarding setup flows
- required_roles: authenticated requester within tenant scope
- required_approvals: none
- required_confirmations: required when permissions are not yet granted
- input_schema (minimum):
```text
tenant_id: string
user_id: string
sms_read_permission_ok: bool
sms_send_permission_ok: bool
setup_source: enum (ONBOARDING | FIRST_SEND_REQUEST | SETTINGS)
idempotency_key: string
```
- output_schema (minimum):
```text
tenant_id: string
user_id: string
sms_app_setup_complete: bool
setup_state: enum (IN_PROGRESS | COMPLETE | BLOCKED)
```
- preconditions: identity verified; user scope valid
- postconditions: setup lifecycle event appended and current setup projection updated
- side_effects: append setup event + update setup current state
- reads_tables[]: [`comms.sms_app_setup_current`]
- writes_tables[]: [`comms.sms_app_setup_ledger`, `comms.sms_app_setup_current`]
- idempotency_key_rule: idempotent on (tenant_id + user_id + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### LEARN_MODEL_UPDATE_SIM (COMMIT)

- name: Learning Artifact Update Commit
- owning_domain: Learning
- simulation_type: COMMIT
- purpose: Commit governed learning artifact updates derived from feedback and correction loops
- triggers: post-send feedback and correction loops
- required_roles: authenticated requester within tenant scope
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
tenant_id: string
user_id: string
feedback_type: enum (DRAFT_CORRECTION | LANGUAGE_USAGE)
feedback_payload_ref: string
idempotency_key: string
```
- output_schema (minimum):
```text
feedback_event_id: string
quality_delta_bucket: string
```
- preconditions: feedback payload exists and belongs to requester scope
- postconditions: feedback event is appended and current adaptation projection updated
- side_effects: append governed learning artifact update event
- reads_tables[]: [`artifacts_ledger`]
- writes_tables[]: [`artifacts_ledger`]
- idempotency_key_rule: idempotent on (tenant_id + user_id + feedback_type + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### MEMORY_FORGET_COMMIT (COMMIT)

- name: Memory Forget Commit
- owning_domain: Memory
- simulation_type: COMMIT
- purpose: Commit a bounded memory forget request for eligible memory artifacts only
- triggers: MEMORY_FORGET_REQUEST (process_id)
- required_roles: authenticated requester within tenant scope
- required_approvals: none
- required_confirmations: required when forget scope is ambiguous
- input_schema (minimum):
```text
tenant_id: string
user_id: string
forget_scope: string
reason_code: string
idempotency_key: string
```
- output_schema (minimum):
```text
forget_result_id: string
forget_outcome: enum (FORGOTTEN | NOOP_ALREADY_FORGOTTEN | PARTIAL_POLICY_BLOCK)
```
- preconditions: identity verified; forget scope belongs to requester; immutable audit/ledger truth excluded from forget scope
- postconditions: eligible memory artifacts are forgotten deterministically within policy boundaries
- side_effects: append forget event + update memory current projection
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (tenant_id + user_id + forget_scope + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### MEMORY_SUPPRESSION_SET_COMMIT (COMMIT)

- name: Memory Suppression Set Commit
- owning_domain: Memory
- simulation_type: COMMIT
- purpose: Commit deterministic suppression control updates (`DO_NOT_MENTION|DO_NOT_REPEAT|DO_NOT_STORE`)
- triggers: MEMORY_FORGET_REQUEST (suppression path)
- required_roles: authenticated requester within tenant scope
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
tenant_id: string
user_id: string
target_type: enum (THREAD_ID | WORK_ORDER_ID | TOPIC_KEY)
target_id: string
rule_kind: enum (DO_NOT_MENTION | DO_NOT_REPEAT | DO_NOT_STORE)
scope: string
reason_code: string
idempotency_key: string
```
- output_schema (minimum):
```text
suppression_rule_id: string
rule_state: enum (SET | UPDATED)
```
- preconditions: identity verified; target scope resolves to requester; rule_kind is policy-allowed
- postconditions: suppression rule is upserted deterministically
- side_effects: upsert suppression rule state
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (tenant_id + user_id + target_type + target_id + rule_kind + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### MEMORY_ATOM_UPSERT_COMMIT (COMMIT)

- name: Memory Atom Upsert Commit
- owning_domain: Memory
- simulation_type: COMMIT
- purpose: Commit one memory atom store/update event for deterministic continuity
- triggers: MEMORY_REMEMBER_REQUEST (process_id)
- required_roles: authenticated requester within tenant scope
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
tenant_id: string
user_id: string
atom_key: string
atom_payload: object
provenance: string
reason_code: string
idempotency_key: string
```
- output_schema (minimum):
```text
atom_event_id: string
atom_state: enum (STORED | UPDATED)
```
- preconditions: identity verified; atom payload is bounded; store policy allows write for the target user scope
- postconditions: atom ledger append and current projection update are committed deterministically
- side_effects: append atom event + update atom current projection
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (tenant_id + user_id + atom_key + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### TOOL_TIME_QUERY_COMMIT (COMMIT)

- name: Tool Time Query Commit
- owning_domain: Tool
- simulation_type: COMMIT
- purpose: Commit read-only TIME lookup outcome as PH1.E audit evidence
- triggers: TOOL_TIME_QUERY (process_id)
- required_roles: authenticated requester within tenant scope
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
tenant_id: string
user_id: string
correlation_id: string
turn_id: string
timezone_hint: string (optional)
query_hash: string
cache_status: string
tool_status: enum (OK | FAIL)
idempotency_key: string
```
- output_schema (minimum):
```text
tool_name: TIME
tool_status: enum (OK | FAIL)
audit_event_id: string
```
- preconditions: identity verified; tool policy allows TIME query execution
- postconditions: tool outcome is recorded deterministically as ToolOk/ToolFail audit evidence
- side_effects: append PH1.E ToolOk/ToolFail audit row only
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (tenant_id + correlation_id + turn_id + query_hash + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### TOOL_WEATHER_QUERY_COMMIT (COMMIT)

- name: Tool Weather Query Commit
- owning_domain: Tool
- simulation_type: COMMIT
- purpose: Commit read-only WEATHER lookup outcome as PH1.E audit evidence
- triggers: TOOL_WEATHER_QUERY (process_id)
- required_roles: authenticated requester within tenant scope
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
tenant_id: string
user_id: string
correlation_id: string
turn_id: string
location: string
start_date: string (optional)
duration_days: integer (optional)
query_hash: string
cache_status: string
tool_status: enum (OK | FAIL)
idempotency_key: string
```
- output_schema (minimum):
```text
tool_name: WEATHER
tool_status: enum (OK | FAIL)
audit_event_id: string
```
- preconditions: identity verified; location is provided; tool policy allows WEATHER query execution
- postconditions: tool outcome is recorded deterministically as ToolOk/ToolFail audit evidence
- side_effects: append PH1.E ToolOk/ToolFail audit row only
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (tenant_id + correlation_id + turn_id + query_hash + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### ONB_BIZ_START_DRAFT (DRAFT)

- name: Business Onboarding Start Draft
- owning_domain: OnboardingBusiness
- simulation_type: DRAFT
- purpose: Start company/tenant prerequisite onboarding draft when employee onboarding requires tenant bootstrap
- triggers: ONB_BIZ_SETUP (process_id)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: none
- required_confirmations: optional (if user asked to proceed now)
- input_schema (minimum):
```text
tenant_id: string
creator_user_id: string
legal_name: string
jurisdiction: string
idempotency_key: string
```
- output_schema (minimum):
```text
company_id: string
lifecycle_state: DRAFT
```
- preconditions: input schema valid + tenant boundary valid + blueprint references simulation + access/confirmation gates satisfied where required
- postconditions: company draft exists deterministically
- side_effects: Write business onboarding draft only
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: required for retriable writes; dedupe on (simulation_id + business_key + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### ONB_BIZ_VALIDATE_COMPANY_COMMIT (COMMIT)

- name: Business Onboarding Validate Company
- owning_domain: OnboardingBusiness
- simulation_type: COMMIT
- purpose: Validate company shell requirements and transition to ACTIVE prerequisite state
- triggers: ONB_BIZ_SETUP (process_id)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: optional AP approval (policy-dependent)
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
company_id: string
tenant_id: string
policy_shell_ref: string
idempotency_key: string
```
- output_schema (minimum):
```text
company_id: string
lifecycle_state: ACTIVE
```
- preconditions: input schema valid + tenant boundary valid + blueprint references simulation + access/confirmation gates satisfied where required
- postconditions: company shell validated and marked ACTIVE for employee onboarding
- side_effects: Create/update tenant company shell
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: required for retriable writes; dedupe on (simulation_id + business_key + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### ONB_BIZ_COMPLETE_COMMIT (COMMIT)

- name: Business Onboarding Complete
- owning_domain: OnboardingBusiness
- simulation_type: COMMIT
- purpose: Mark business onboarding prerequisite complete and emit handoff-ready state
- triggers: ONB_BIZ_SETUP (process_id)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: none
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
company_id: string
tenant_id: string
idempotency_key: string
```
- output_schema (minimum):
```text
company_id: string
business_onboarding_status: COMPLETE
```
- preconditions: input schema valid + tenant boundary valid + blueprint references simulation + access/confirmation gates satisfied where required
- postconditions: business onboarding prerequisite complete
- side_effects: Mark business onboarding complete
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: required for retriable writes; dedupe on (simulation_id + business_key + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### POSITION_SIM_001_CREATE_DRAFT (DRAFT)

- name: Position Create Draft
- owning_domain: Position
- simulation_type: DRAFT
- purpose: Create deterministic position draft for a verified company
- triggers: POSITION_MANAGE (process_id)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: none
- required_confirmations: none (DRAFT)
- input_schema (minimum):
```text
tenant_id: string
company_id: string
position_title: string
department: string
jurisdiction: string
schedule_type: enum
permission_profile_ref: string
compensation_band_ref: string
idempotency_key: string
```
- output_schema (minimum):
```text
position_id: string
lifecycle_state: Draft
```
- preconditions: input schema valid + tenant boundary valid + blueprint references simulation + access/confirmation gates satisfied where required
- postconditions: position draft row exists
- side_effects: Write position draft row
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: required for retriable writes; dedupe on (simulation_id + business_key + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### POSITION_SIM_002_VALIDATE_AUTH_COMPANY (DRAFT)

- name: Position Validate Authority And Company
- owning_domain: Position
- simulation_type: DRAFT
- purpose: Validate company existence/verification and authority scope before lifecycle transition
- triggers: POSITION_MANAGE (process_id)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
tenant_id: string
company_id: string
position_id: string
requested_action: enum (ACTIVATE | SUSPEND | RETIRE)
```
- output_schema (minimum):
```text
validation_status: enum (OK | FAIL)
reason_code: string
```
- preconditions: input schema valid + tenant boundary valid + blueprint references simulation + access/confirmation gates satisfied where required
- postconditions: deterministic validation result exists
- side_effects: Write validation result only
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: required for retriable writes; dedupe on (simulation_id + business_key + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### POSITION_SIM_003_BAND_POLICY_CHECK (DRAFT)

- name: Position Compensation Band Policy Check
- owning_domain: Position
- simulation_type: DRAFT
- purpose: Enforce compensation-band policy and emit AP escalation requirement when needed
- triggers: POSITION_MANAGE (process_id)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: optional AP approval (policy-dependent)
- required_confirmations: none
- input_schema (minimum):
```text
tenant_id: string
position_id: string
compensation_band_ref: string
```
- output_schema (minimum):
```text
policy_result: enum (ALLOW | ESCALATE | DENY)
reason_code: string
```
- preconditions: input schema valid + tenant boundary valid + blueprint references simulation + access/confirmation gates satisfied where required
- postconditions: policy decision recorded deterministically
- side_effects: Write policy check result only
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: required for retriable writes; dedupe on (simulation_id + business_key + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### POSITION_SIM_004_ACTIVATE_COMMIT (COMMIT)

- name: Position Activate
- owning_domain: Position
- simulation_type: COMMIT
- purpose: Activate a position draft
- triggers: POSITION_MANAGE (process_id)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: optional AP approval (policy-dependent)
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
tenant_id: string
position_id: string
idempotency_key: string
```
- output_schema (minimum):
```text
position_id: string
lifecycle_state: Active
```
- preconditions: input schema valid + tenant boundary valid + blueprint references simulation + access/confirmation gates satisfied where required
- postconditions: position lifecycle transitions to Active and lifecycle event is appended
- side_effects: Update position lifecycle to Active
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: required for retriable writes; dedupe on (simulation_id + business_key + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### POSITION_SIM_005_RETIRE_OR_SUSPEND_COMMIT (COMMIT)

- name: Position Retire Or Suspend
- owning_domain: Position
- simulation_type: COMMIT
- purpose: Retire or suspend an active position deterministically
- triggers: POSITION_MANAGE (process_id)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: optional AP approval (policy-dependent)
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
tenant_id: string
position_id: string
requested_state: enum (Suspended | Retired)
idempotency_key: string
```
- output_schema (minimum):
```text
position_id: string
lifecycle_state: enum (Suspended | Retired)
```
- preconditions: input schema valid + tenant boundary valid + blueprint references simulation + access/confirmation gates satisfied where required
- postconditions: position lifecycle transitions to requested terminal/intermediate state with lifecycle event append
- side_effects: Update position lifecycle state
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: required for retriable writes; dedupe on (simulation_id + business_key + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### POSITION_REQUIREMENTS_SCHEMA_CREATE_DRAFT (DRAFT)

- name: Position Requirements Schema Create Draft
- owning_domain: Position
- simulation_type: DRAFT
- purpose: Create a deterministic requirements schema draft bound to a position
- triggers: ONB_SCHEMA_MANAGE (process_id)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: none
- required_confirmations: none (DRAFT)
- input_schema (minimum):
```text
tenant_id: string
position_id: string
schema_change_set: object (bounded)
selector_scope: object (bounded)
idempotency_key: string
```
- output_schema (minimum):
```text
schema_id: string
schema_version: string
schema_status: enum (DRAFT)
```
- preconditions: position exists and is tenant-scoped; caller is authorized
- postconditions: requirements schema draft exists for position
- side_effects: Write position requirements schema draft
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (tenant_id + position_id + schema_change_set_hash + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### POSITION_REQUIREMENTS_SCHEMA_UPDATE_COMMIT (COMMIT)

- name: Position Requirements Schema Update Commit
- owning_domain: Position
- simulation_type: COMMIT
- purpose: Apply deterministic field/overlay operations to an existing position requirements schema draft
- triggers: ONB_SCHEMA_MANAGE (process_id)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: optional AP approval (policy-dependent)
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
tenant_id: string
position_id: string
schema_id: string
schema_version: string
overlay_ops: object (bounded)
idempotency_key: string
```
- output_schema (minimum):
```text
schema_id: string
schema_version: string
schema_status: enum (DRAFT)
```
- preconditions: schema draft exists; overlay operations validate deterministically
- postconditions: schema draft is updated deterministically with replay-safe diff
- side_effects: Write requirements schema update
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (tenant_id + position_id + schema_id + schema_version + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### POSITION_REQUIREMENTS_SCHEMA_ACTIVATE_COMMIT (COMMIT)

- name: Position Requirements Schema Activate Commit
- owning_domain: Position
- simulation_type: COMMIT
- purpose: Activate a position requirements schema version for deterministic onboarding use
- triggers: ONB_SCHEMA_MANAGE (process_id)
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: optional AP approval (policy-dependent)
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
tenant_id: string
position_id: string
schema_id: string
schema_version: string
apply_scope: enum (NewHiresOnly | CurrentAndNew)
idempotency_key: string
```
- output_schema (minimum):
```text
schema_id: string
schema_version: string
schema_status: enum (ACTIVE)
apply_scope_result: enum (NewHiresOnly | CurrentAndNew)
backfill_handoff_required: bool
```
- preconditions: schema draft validates; rollout scope explicitly confirmed
- postconditions: active requirements schema for position is updated deterministically; `CurrentAndNew` activation requires explicit backfill handoff context
- side_effects: Update active schema version for position
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: idempotent on (tenant_id + position_id + schema_id + schema_version + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### TENANT_CONTEXT_RESOLVE_DRAFT (DRAFT)

- name: Tenant Context Resolve
- owning_domain: Tenant
- simulation_type: DRAFT
- purpose: Resolve tenant context and policy snapshot before enterprise execution
- triggers: ENTERPRISE_PRELUDE
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
tenant_hint: string
actor_user_id: string
```
- output_schema (minimum):
```text
tenant_id: string
policy_snapshot_ref: string
```
- preconditions: input schema valid + tenant boundary valid + blueprint references simulation + access/confirmation gates satisfied where required
- postconditions: deterministic tenant context resolution event exists
- side_effects: Write context resolution result only
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: required for retriable writes; dedupe on (simulation_id + business_key + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### QUOTA_CHECK_DRAFT (DRAFT)

- name: Quota Check
- owning_domain: Quota
- simulation_type: DRAFT
- purpose: Evaluate deterministic budget/quota gates before expensive calls
- triggers: ENTERPRISE_PRELUDE
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
tenant_id: string
user_id: string
resource_class: string
requested_units: int
```
- output_schema (minimum):
```text
quota_decision: enum (ALLOW | WAIT | REFUSE)
reason_code: string
```
- preconditions: input schema valid + tenant boundary valid + blueprint references simulation + access/confirmation gates satisfied where required
- postconditions: deterministic quota decision exists
- side_effects: Write quota decision result only
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: required for retriable writes; dedupe on (simulation_id + business_key + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### KMS_HANDLE_ISSUE_COMMIT (COMMIT)

- name: KMS Handle Issue
- owning_domain: KMS
- simulation_type: COMMIT
- purpose: Issue a short-lived credential handle for approved runtime use
- triggers: ENTERPRISE_PRELUDE
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: optional (policy-dependent)
- required_confirmations: none
- input_schema (minimum):
```text
tenant_id: string
credential_class: string
ttl_ms: int
idempotency_key: string
```
- output_schema (minimum):
```text
kms_handle_ref: string
expires_at_ms: timestamp_ms
```
- preconditions: input schema valid + tenant boundary valid + blueprint references simulation + access/confirmation gates satisfied where required
- postconditions: short-lived handle issuance event exists
- side_effects: Write handle issue/audit row
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: required for retriable writes; dedupe on (simulation_id + business_key + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### WORK_ORDER_APPEND_COMMIT (COMMIT)

- name: Work Order Append
- owning_domain: WorkOrder
- simulation_type: COMMIT
- purpose: Append deterministic work order event to ledger
- triggers: OS_RUNTIME
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
work_order_id: string
event_type: string
payload_hash: string
idempotency_key: string
```
- output_schema (minimum):
```text
work_order_event_id: string
```
- preconditions: input schema valid + tenant boundary valid + blueprint references simulation + access/confirmation gates satisfied where required
- postconditions: work order event appended
- side_effects: Append work_order_ledger row
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: required for retriable writes; dedupe on (simulation_id + business_key + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### WORK_LEASE_ACQUIRE_COMMIT (COMMIT)

- name: Work Lease Acquire
- owning_domain: WorkLease
- simulation_type: COMMIT
- purpose: Acquire deterministic lease for work order execution ownership
- triggers: OS_RUNTIME
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
work_order_id: string
executor_id: string
lease_ttl_ms: int
idempotency_key: string
```
- output_schema (minimum):
```text
lease_id: string
lease_status: enum (ACQUIRED | DENIED)
```
- preconditions: input schema valid + tenant boundary valid + blueprint references simulation + access/confirmation gates satisfied where required
- postconditions: lease state updated deterministically
- side_effects: Write work_order_leases row
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: required for retriable writes; dedupe on (simulation_id + business_key + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### WORK_LEASE_RENEW_COMMIT (COMMIT)

- name: Work Lease Renew
- owning_domain: WorkLease
- simulation_type: COMMIT
- purpose: Renew deterministic lease for in-flight work order execution
- triggers: OS_RUNTIME
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
lease_id: string
work_order_id: string
executor_id: string
lease_ttl_ms: int
idempotency_key: string
```
- output_schema (minimum):
```text
lease_id: string
lease_status: enum (RENEWED | DENIED)
```
- preconditions: input schema valid + tenant boundary valid + blueprint references simulation + access/confirmation gates satisfied where required
- postconditions: lease renewal state updated deterministically
- side_effects: Update work_order_leases row
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: required for retriable writes; dedupe on (simulation_id + business_key + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### WORK_LEASE_RELEASE_COMMIT (COMMIT)

- name: Work Lease Release
- owning_domain: WorkLease
- simulation_type: COMMIT
- purpose: Release deterministic lease after work order step completion/failure
- triggers: OS_RUNTIME
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
lease_id: string
work_order_id: string
executor_id: string
idempotency_key: string
```
- output_schema (minimum):
```text
lease_id: string
lease_status: RELEASED
```
- preconditions: input schema valid + tenant boundary valid + blueprint references simulation + access/confirmation gates satisfied where required
- postconditions: lease released deterministically
- side_effects: Update work_order_leases row
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: required for retriable writes; dedupe on (simulation_id + business_key + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### SCHED_NEXT_ACTION_DRAFT (DRAFT)

- name: Scheduler Next Action
- owning_domain: Scheduler
- simulation_type: DRAFT
- purpose: Compute deterministic next retry/timeout action
- triggers: OS_RUNTIME
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
work_order_id: string
step_id: string
retry_count: int
policy_ref: string
```
- output_schema (minimum):
```text
next_action: enum (WAIT | RETRY | FAIL)
next_due_at_ms: timestamp_ms
```
- preconditions: input schema valid + tenant boundary valid + blueprint references simulation + access/confirmation gates satisfied where required
- postconditions: deterministic scheduler decision record exists
- side_effects: Write scheduler decision result only
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: required for retriable writes; dedupe on (simulation_id + business_key + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### GOV_ACTIVATE_DEFINITION_COMMIT (COMMIT)

- name: Governance Activate Definition
- owning_domain: Governance
- simulation_type: COMMIT
- purpose: Activate signed blueprint/simulation/capability definitions
- triggers: GOV_ADMIN
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: required (governance policy)
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
definition_pack_id: string
target_scope: string
signature_ref: string
idempotency_key: string
```
- output_schema (minimum):
```text
definition_pack_id: string
activation_status: ACTIVE
```
- preconditions: input schema valid + tenant boundary valid + blueprint references simulation + access/confirmation gates satisfied where required
- postconditions: governance definition activation state updated
- side_effects: Update governance_definitions state
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: required for retriable writes; dedupe on (simulation_id + business_key + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### GOV_ROLLBACK_DEFINITION_COMMIT (COMMIT)

- name: Governance Rollback Definition
- owning_domain: Governance
- simulation_type: COMMIT
- purpose: Roll back active definitions to prior signed version
- triggers: GOV_ADMIN
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: required (governance policy)
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
active_definition_pack_id: string
rollback_to_pack_id: string
idempotency_key: string
```
- output_schema (minimum):
```text
rollback_status: ACTIVE
```
- preconditions: input schema valid + tenant boundary valid + blueprint references simulation + access/confirmation gates satisfied where required
- postconditions: governance definition rollback state updated
- side_effects: Update governance_definitions state
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: required for retriable writes; dedupe on (simulation_id + business_key + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### EXPORT_BUILD_PACK_COMMIT (COMMIT)

- name: Export Build Pack
- owning_domain: Export
- simulation_type: COMMIT
- purpose: Build a tamper-evident compliance export package with policy-safe redaction
- triggers: EXPORT_REQUEST
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: optional (policy-dependent)
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
tenant_id: string
export_scope: string
date_range: string
idempotency_key: string
```
- output_schema (minimum):
```text
export_job_id: string
artifact_ref: string
status: COMPLETED
```
- preconditions: input schema valid + tenant boundary valid + blueprint references simulation + access/confirmation gates satisfied where required
- postconditions: export job row and artifact reference exist
- side_effects: Write export_jobs row + artifact ref
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: required for retriable writes; dedupe on (simulation_id + business_key + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### REVIEW_ROUTE_CASE_DRAFT (DRAFT)

- name: Review Route Case
- owning_domain: Governance
- simulation_type: DRAFT
- purpose: Route policy-required human review case before commit
- triggers: REVIEW_REQUIRED
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: none
- required_confirmations: none
- input_schema (minimum):
```text
case_type: string
requested_action_ref: string
requester_user_id: string
```
- output_schema (minimum):
```text
review_case_id: string
status: OPEN
```
- preconditions: input schema valid + tenant boundary valid + blueprint references simulation + access/confirmation gates satisfied where required
- postconditions: review case draft exists deterministically
- side_effects: Write review_cases draft row
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: required for retriable writes; dedupe on (simulation_id + business_key + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]

### REVIEW_DECISION_COMMIT (COMMIT)

- name: Review Decision Commit
- owning_domain: Governance
- simulation_type: COMMIT
- purpose: Commit human review decision (approve/reject) with deterministic reason codes
- triggers: REVIEW_DECISION
- required_roles: POLICY_ROLE_BOUND (resolved by PH1.ACCESS.001 -> PH2.ACCESS.002 + blueprint policy)
- required_approvals: required (human review decision)
- required_confirmations: required (COMMIT)
- input_schema (minimum):
```text
review_case_id: string
decision: enum (APPROVE | REJECT)
decider_user_id: string
idempotency_key: string
```
- output_schema (minimum):
```text
review_case_id: string
status: enum (APPROVED | REJECTED)
```
- preconditions: input schema valid + tenant boundary valid + blueprint references simulation + access/confirmation gates satisfied where required
- postconditions: review case decision state committed deterministically
- side_effects: Update review_cases decision state
- reads_tables[]: inherited from owning_domain profile (or stricter record override)
- writes_tables[]: inherited from owning_domain profile (or stricter record override)
- idempotency_key_rule: required for retriable writes; dedupe on (simulation_id + business_key + idempotency_key)
- audit_events: [SIMULATION_STARTED, SIMULATION_FINISHED, SIMULATION_REASON_CODED]
