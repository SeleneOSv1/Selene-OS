# Selene PH1.BCAST / PH1.DELIVERY / PH1.REM — Repo-Truth Functionality Extraction Master Design

DOCUMENT STATUS:
REPO_TRUTH_EXTRACTION
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

## 0. Authority and Scope

AGENTS.md controls execution.

This is a docs-only repo-truth extraction.

No runtime code was changed.

This document does not authorize implementation.

This document reconstructs current broadcast, delivery, and reminder design/functionality from repo evidence.

Future implementation, refactor, or retirement requires explicit build instruction, approved file scope, tests, backend evidence, JD approval, and where visible JD live proof.

This extraction is intentionally conservative. When repo evidence is incomplete, this document marks the area as `NOT_FOUND`, `PARTIAL`, `UNKNOWN`, `REPO_TRUTH_NEEDED`, `DESIGN_GAP`, `TEST_GAP`, `OWNER_GAP`, or `AUDIT_GAP`.

## 1. Executive Summary

PH1.BCAST, PH1.DELIVERY, and PH1.REM are currently related but separate Selene owners.

PH1.BCAST is the authoritative broadcast lifecycle owner for draft, delivery request, acknowledgement, defer, reminder-fired resume, escalation, expiration, and cancellation. The active implementation id is `PH1.BCAST.001`. Current Rust runtime evidence is in `crates/selene_kernel_contracts/src/ph1bcast.rs`, `crates/selene_engines/src/ph1bcast.rs`, `crates/selene_os/src/ph1bcast.rs`, `crates/selene_os/src/simulation_executor.rs`, and PH1.F storage projection code in `crates/selene_storage/src/ph1f.rs`.

PH1.DELIVERY is the authoritative delivery-attempt truth owner. It normalizes provider send, status, cancel, and provider health check behavior for channels including `Sms`, `Email`, `Whatsapp`, `Wechat`, and `AppPush`. Current runtime evidence is deterministic and provider-simulated: provider refs and KMS handle refs exist, but no live Twilio, SendGrid, WhatsApp, WeChat, APNS, or SMTP provider integration was found in the inspected repo surfaces.

PH1.REM is the authoritative reminder timing state machine. It schedules, updates, cancels, snoozes, schedules follow-ups, schedules delivery retry, records pre/due delivery attempts, escalates, and marks reminder occurrences completed or failed. The current runtime is not a standalone `crates/selene_engines/src/ph1rem.rs` engine. Instead, PH1.REM is wired through `crates/selene_os/src/ph1rem.rs` into PH1.F storage via `Ph1fStore::ph1rem_run`.

The three owners are not one combined engine:

- PH1.BCAST owns message/broadcast recipient lifecycle.
- PH1.DELIVERY owns provider-attempt execution/proof only.
- PH1.REM owns timing mechanics only.
- Selene OS orchestrates bridges between them.

Product functions currently evidenced include:

- broadcast draft creation,
- one-recipient delivery commit requests,
- Selene App first delivery posture,
- fallback order gating for SMS, WhatsApp, WeChat, and Email when app delivery is unavailable,
- link invite delivery through `LINK_DELIVER_INVITE`,
- message compose and send blueprint routing,
- reminder scheduling/update/cancel/snooze/follow-up/retry/deliver/escalate/complete/fail,
- BCAST.MHP phone-first follow-up behavior,
- app-thread reply auto-forward proof path,
- access-gated reminder creation/update/cancel dispatch,
- BCAST policy updates for non-urgent wait and urgent follow-up behavior,
- append-only delivery and reminder delivery attempt evidence.

Current active evidence is strongest for deterministic contracts, in-memory runtime behavior, OS bridge orchestration, PH1.F persistence/projection, and tests. Current evidence is weaker or missing for durable SQL migrations for comms/reminders, live external provider integrations, broad multi-recipient audience resolution runtime, PH1.WRITE-owned final message drafting integration, live client notification push, and user-visible delivery status UX beyond read-only operational surfaces.

## 2. Repo Evidence Inventory

| Evidence Area | File / Path | Symbols / Routes / Tables | Status | Notes |
|---|---|---|---|---|
| BCAST kernel contract | `crates/selene_kernel_contracts/src/ph1bcast.rs` | `Ph1BcastRequest`, `BcastRequest`, `BcastDraftCreateRequest`, `BcastDeliverCommitRequest`, `BcastDeferCommitRequest`, `BcastReminderFiredCommitRequest`, `BcastAckCommitRequest`, `BcastEscalateCommitRequest`, `BcastExpireCommitRequest`, `BcastCancelCommitRequest`, `BcastOutcome`, `BcastRecipientState`, `BcastDeliveryMethod`, `BCAST_CREATE_DRAFT`, `BCAST_DELIVER_COMMIT`, `BCAST_DEFER_COMMIT`, `BCAST_REMINDER_FIRED_COMMIT`, `BCAST_ACK_COMMIT`, `BCAST_ESCALATE_COMMIT`, `BCAST_EXPIRE_COMMIT`, `BCAST_CANCEL_COMMIT` | FOUND | Contract enforces simulation id/type matching, fallback proof fields, no-authority/simulation-gated safety flags in OK response. |
| BCAST runtime | `crates/selene_engines/src/ph1bcast.rs` | `Ph1BcastRuntime`, `Ph1BcastConfig`, `reason_codes::*`, `run_draft_create`, `run_deliver`, `run_defer`, `run_reminder_fired`, `run_ack`, `run_escalate`, `run_expire`, `run_cancel` | FOUND | In-memory deterministic lifecycle runtime; no direct PH1.DELIVERY call. |
| BCAST OS wiring | `crates/selene_os/src/ph1bcast.rs` | `Ph1BcastWiring`, `Ph1BcastEngine`, `Ph1BcastWiringConfig`, `Ph1BcastDispatchOutcome` | FOUND | Validates request/response, supports disabled wiring, fails closed on simulation/capability drift. |
| DELIVERY kernel contract | `crates/selene_kernel_contracts/src/ph1delivery.rs` | `Ph1DeliveryRequest`, `DeliveryRequest`, `DeliverySendRequest`, `DeliveryStatusRequest`, `DeliveryCancelRequest`, `DeliveryProviderHealthCheckRequest`, `DeliveryOutcome`, `DeliveryChannel`, `DeliveryStatus`, `DELIVERY_SEND_COMMIT`, `DELIVERY_CANCEL_COMMIT`, `DELIVERY_STATUS_DRAFT`, `DELIVERY_PROVIDER_HEALTH_CHECK_DRAFT` | FOUND | Contract separates commit send/cancel from draft status/health. |
| DELIVERY runtime | `crates/selene_engines/src/ph1delivery.rs` | `Ph1DeliveryRuntime`, `Ph1DeliveryConfig`, `DeliveryProviderBinding`, `default_provider_ref_for_channel`, deterministic provider refs and KMS handle refs | FOUND | Deterministic provider simulation, not live provider integration. |
| DELIVERY OS wiring | `crates/selene_os/src/ph1delivery.rs` | `Ph1DeliveryWiring`, `Ph1DeliveryEngine`, `Ph1DeliveryWiringConfig`, `Ph1DeliveryDispatchOutcome` | FOUND | Validates request/response, disabled path, drift fail-closed, request contract validation bubble. |
| REM kernel contract | `crates/selene_kernel_contracts/src/ph1rem.rs` | `Ph1RemRequest`, `ReminderRequest`, `ReminderScheduleCommitRequest`, `ReminderUpdateCommitRequest`, `ReminderCancelCommitRequest`, `ReminderSnoozeCommitRequest`, `ReminderFollowupScheduleCommitRequest`, `ReminderDeliveryRetryScheduleCommitRequest`, `ReminderDeliverPreCommitRequest`, `ReminderDeliverDueCommitRequest`, `ReminderEscalateCommitRequest`, `ReminderMarkCompletedCommitRequest`, `ReminderMarkFailedCommitRequest`, `ReminderType`, `ReminderState`, `ReminderChannel`, `ReminderDeliveryStatus`, `REMINDER_*` simulation ids | FOUND | Full contract for reminder timing and delivery-attempt proof. |
| REM engine runtime file | `crates/selene_engines/src/ph1rem.rs` | None | NOT_FOUND | Current PH1.REM runtime evidence lives in OS + PH1.F storage, not a separate engine file. |
| REM OS runtime | `crates/selene_os/src/ph1rem.rs` | `Ph1RemRuntime`, `run_for_implementation`, `REM_FAIL_TIME_AMBIGUOUS_NEEDS_CONFIRM` | FOUND | Calls `Ph1fStore::ph1rem_run`; implementation id locked to `PH1.REM.001`. |
| OS simulation bridge | `crates/selene_os/src/simulation_executor.rs` | `run_broadcast_deliver_with_delivery`, `run_broadcast_mhp_defer_with_reminder`, `run_broadcast_mhp_mark_reminder_fired`, `run_broadcast_mhp_app_thread_reply_conclude`, `execute_rem`, `is_legacy_link_delivery_simulation_id`, `SimulationDispatchOutcome::BroadcastDeliverySend`, `LinkDelivered`, `Reminder` | FOUND | Canonical orchestration bridge; BCAST -> DELIVERY and BCAST -> REM ownership split is explicit. |
| PH1.F BCAST persistence | `crates/selene_storage/src/ph1f.rs` | `BcastRecipientLifecycleLedgerRow`, `BcastBroadcastCurrentRecord`, `BcastRecipientLifecycleCurrentRecord`, `append_bcast_recipient_lifecycle_event`, `rebuild_bcast_recipient_lifecycle_current_from_ledger`, `attempt_overwrite_bcast_recipient_lifecycle_event` | FOUND | Append-only lifecycle ledger and rebuildable current projections. |
| PH1.F DELIVERY persistence | `crates/selene_storage/src/ph1f.rs` | `DeliveryAttemptLedgerRow`, `DeliveryAttemptCurrentRecord`, `append_delivery_attempt_event`, `rebuild_delivery_attempts_current_from_ledger`, `attempt_overwrite_delivery_attempt_event`, `delivery_send_response_by_idempotency` | FOUND | Append-only provider attempt ledger and current projection for send attempts. |
| PH1.F REM persistence | `crates/selene_storage/src/ph1f.rs` | `ReminderRecord`, `ReminderOccurrenceRecord`, `ReminderDeliveryAttemptRecord`, `ph1rem_run`, `reminders`, `reminder_occurrences`, `reminder_delivery_attempts`, `rebuild_reminder_delivery_attempt_indexes` | FOUND | Current reminder records and append-only reminder delivery attempt records. |
| SQL migrations | `crates/selene_storage/migrations/*.sql` | Search found no broadcast/delivery/reminder tables | NOT_FOUND | PH1.F in-memory/storage module has structs and behavior; no current SQL migration defining comms/reminder tables was found. |
| BCAST DB wiring doc | `docs/DB_WIRING/PH1_BCAST.md` | `comms.broadcast_envelopes_ledger`, `comms.broadcast_envelopes_current`, `comms.broadcast_recipients_current`, `comms.broadcast_delivery_attempts_ledger`, `comms.broadcast_ack_ledger`, BCAST.MHP | FOUND / PARTIAL | Design declares comms tables and MHP rules; current code uses PH1.F structs rather than SQL table migrations. |
| DELIVERY DB wiring doc | `docs/DB_WIRING/PH1_DELIVERY.md` | `comms.delivery_attempts_ledger`, `comms.delivery_attempts_current`, `comms.delivery_provider_health` | FOUND / PARTIAL | Design declares provider-attempt tables and secrets-as-KMS-handles rule; current runtime uses deterministic provider refs and PH1.F ledger. |
| REM DB wiring doc | `docs/DB_WIRING/PH1_REM.md` | `reminders`, `reminder_occurrences`, `reminder_delivery_attempts`, BCAST.MHP timing split | FOUND / PARTIAL | Doc claims design-only implementation in places, but repo now contains runtime/storage behavior. Mark stale doc note. |
| BCAST ECM | `docs/ECM/PH1_BCAST.md` | `BCAST_DRAFT_CREATE`, `BCAST_AUDIENCE_RESOLVE`, `BCAST_PRIVACY_HANDSHAKE`, `BCAST_DELIVER_COMMIT`, `BCAST_ACK_RECORD`, `BCAST_DEFER_AND_SCHEDULE_RETRY`, `BCAST_REMINDER_FIRED`, `BCAST_ESCALATE_TO_SENDER`, `BCAST_CANCEL`, `BCAST_EXPIRE` | FOUND / PARTIAL | Audience/privacy capabilities are documented but not current runtime request variants. |
| DELIVERY ECM | `docs/ECM/PH1_DELIVERY.md` | `DELIVERY_SEND`, `DELIVERY_STATUS`, `DELIVERY_CANCEL`, `DELIVERY_PROVIDER_HEALTH_CHECK` | FOUND | Matches current delivery contract/runtime. |
| REM ECM | `docs/ECM/PH1_REM.md` | `PH1REM_*_COMMIT_ROW` capabilities | FOUND | Matches current REM contract categories. |
| Simulation catalog | `docs/08_SIMULATION_CATALOG.md` | `BCAST_*`, `DELIVERY_*`, `REMINDER_*`, legacy LINK delivery `LEGACY_DO_NOT_WIRE` | FOUND | BCAST modern ids marked `DRAFT` in status column while runtime treats them as active contract ids. Mark status mismatch as PARTIAL. |
| Blueprint: message send | `docs/BLUEPRINTS/MESSAGE_COMPOSE_AND_SEND.md` | `MESSAGE_COMPOSE_AND_SEND`, `MSG_S09` PH1.BCAST draft, `MSG_S10` BCAST deliver, `MSG_S11` PH1.DELIVERY send | FOUND | Defines message/email compose/send flow with confirmation, access gate, BCAST, DELIVERY. |
| Blueprint: link delivery | `docs/BLUEPRINTS/LINK_DELIVER_INVITE.md` | `LINK_DELIVER_INVITE`, `LINK_DELIVER_S03` BCAST, `LINK_DELIVER_S04` BCAST deliver, `LINK_DELIVER_S05` DELIVERY send | FOUND | Current canonical invite delivery orchestration. |
| Blueprint: reminder manage | `docs/BLUEPRINTS/REMINDER_MANAGE.md` | `REMINDER_MANAGE`, `REM_S02`..`REM_S13` | FOUND | Defines REM lifecycle and BCAST handoff for `BCAST_MHP_FOLLOWUP`. |
| Blueprint: link invite | `docs/BLUEPRINTS/LINK_INVITE.md` | Delivery note: PH1.LINK does not send; delivery via `LINK_DELIVER_INVITE` | FOUND | Confirms PH1.LINK and delivery separation. |
| Blueprint: onboarding backfill | `docs/BLUEPRINTS/ONB_REQUIREMENT_BACKFILL.md` | `ONB_BACKFILL_S06` BCAST, `S07` BCAST deliver, `S08` REM, `S09` ONB notify | FOUND | Shows ONB owns campaign progress while BCAST/REM handle notification and timing handoffs. |
| OS DB wiring | `docs/DB_WIRING/PH1_OS.md` | Outbound delivery ownership lock, `AT-OS-15`, `AT-OS-16` | FOUND | Explicitly says PH1.LINK token/draft only, BCAST lifecycle, DELIVERY provider-attempt proof only, REM timing only. |
| Access DB wiring | `docs/DB_WIRING/PH1_ACCESS_001_PH2_ACCESS_002.md` | SMS setup gates, AP escalation via BCAST | FOUND | Access can block/require setup before SMS send; Access does not call BCAST directly. |
| Runtime guard script | `scripts/check_delivery_ownership_boundaries.sh` | `CHECK_OK delivery_ownership_boundaries=pass`, required/absent patterns | FOUND | Static guard prevents legacy LINK delivery re-entry and direct engine ownership drift. |
| Adapter runtime | `crates/selene_adapter/src/lib.rs`, `crates/selene_adapter/src/bin/http_adapter.rs` | Reminder dispatch tests, dead-letter/System Activity/Needs Attention support, no broad delivery provider route found | PARTIAL | Adapter has operational visibility and reminder dispatch tests; no external provider ownership found. |
| Desktop client | `apple/mac_desktop/SeleneMacDesktop/*.swift` | `System Activity`, `Needs Attention`, read-only pending/failed messaging, onboarding submit surfaces | PARTIAL | Desktop renders operational state and sends user/runtime turns; no external delivery authority found. |
| iPhone client | `apple/iphone/SeleneIPhone/SessionShellView.swift` | `broadcast_waiting_followup_reminder_state`, `dead_letter_or_failed_delivery`, `broadcast_followup_requires_human_action` | PARTIAL | iPhone exposes read-only operational visibility; no local send/authority found. |
| Tool lane negative test | `crates/selene_tools/src/ph1e.rs` | hostile `send_email` rejected as forbidden tool | FOUND | PH1.E remains read-only and cannot become delivery owner. |
| Live provider integrations | repo search for Twilio, SendGrid, SMTP, APNS, Firebase delivery provider code | NOT_FOUND | Provider refs exist as deterministic strings only; no live outbound provider implementation found. |

## 3. Current Owner Map

| Responsibility | Current Owner / File | Correct Future Owner Hypothesis | Status | Notes |
|---|---|---|---|---|
| broadcast draft creation | PH1.BCAST / `crates/selene_engines/src/ph1bcast.rs`, contract in `ph1bcast.rs` | PH1.BCAST | FOUND | Creates `broadcast_id`, stores classification and payload ref; content itself is reference-based. |
| broadcast audience resolution | ECM design only / `docs/ECM/PH1_BCAST.md` | PH1.BCAST with Access/Governance and PH1.X inputs | PARTIAL | `BCAST_AUDIENCE_RESOLVE` is documented but no current runtime request variant was found. |
| message body/template creation | `MESSAGE_COMPOSE_AND_SEND` blueprint, PH1.X clarify fields, BCAST `content_payload_ref` | PH1.WRITE should own final wording; BCAST should carry payload refs | PARTIAL | Current BCAST runtime does not write message text; future boundary should prevent BCAST from inventing wording. |
| final wording handoff from PH1.WRITE | Not directly evidenced in BCAST runtime | PH1.WRITE | DESIGN_GAP | `BCAST_WRITING_OWNER_RISK` if future runtime lets BCAST create content rather than receiving approved payload refs. |
| recipient/contact resolution | `BcastRecipientId`, `recipient`, `recipient_contact`, PH1.X helpers, blueprints | PH1.X + Access/recipient resolver + BCAST recipient lifecycle | PARTIAL | Simple bounded strings/ids exist; robust resolver/contact consent is underdefined. |
| delivery channel selection | `BcastDeliveryMethod`, `DeliveryChannel`, PH1.X field prompt, simulation executor mapping | PH1.X validates request; BCAST/DELIVERY enforce channel/fallback | FOUND | BCAST fallback has app-unavailable proof rules; DELIVERY maps channel to provider binding. |
| SMS delivery | PH1.DELIVERY runtime deterministic provider binding | PH1.DELIVERY plus provider governance | PARTIAL | `provider:sms/default` and `kms://delivery/sms/default`; no live SMS provider proof. |
| email delivery | PH1.DELIVERY runtime deterministic provider binding | PH1.DELIVERY plus provider governance | PARTIAL | `provider:email/default`; no SMTP/SendGrid proof. |
| WhatsApp/WeChat delivery | PH1.DELIVERY runtime deterministic provider binding; BCAST fallback | PH1.DELIVERY plus channel policy | PARTIAL | Region-sensitive fallback; no live provider proof. |
| in-app/push delivery | `DeliveryChannel::AppPush`, BCAST `SeleneApp` maps to AppPush in simulation executor | PH1.DELIVERY + client render only | PARTIAL | `provider:app_push/default`; no live APNS/push provider proof found. |
| reminder scheduling | PH1.REM via `Ph1fStore::ph1rem_schedule` | PH1.REM | FOUND | Deterministic time parsing supports bounded forms; ambiguous time fails closed. |
| reminder firing | PH1.REM `DeliverPreCommit`/`DeliverDueCommit`; OS resumes BCAST for MHP | PH1.REM timing, PH1.BCAST lifecycle | FOUND | Due delivery can set follow-up pending and retry scheduling. |
| resend/retry/failure handling | BCAST defer/escalate/expire; REM retry schedule; DELIVERY fail reason codes | PH1.BCAST + PH1.REM + PH1.DELIVERY | PARTIAL | Deterministic retry states exist; live provider retries/failover not found. |
| delivery status tracking | PH1.DELIVERY status request/result; PH1.F current projection for send attempts | PH1.DELIVERY | PARTIAL | Contract/runtime support status normalization; PH1.F append path currently accepts send outcome only. |
| delivery cancellation | PH1.DELIVERY cancel contract/runtime | PH1.DELIVERY | PARTIAL | Runtime supports cancel for SMS/Email/AppPush, not WhatsApp/WeChat; persistence append function is send-specific. |
| delivery audit/provenance | PH1.F delivery ledger/current; DB docs; proof refs | PH1.DELIVERY + PH1.J future audit | PARTIAL | PH1.F records requests/responses/proof refs; PH1.J audit linkage not fully evidenced in runtime path. |
| link invite delivery | Simulation executor `IntentType::CreateInviteLink` plus `LINK_DELIVER_INVITE` blueprint | PH1.LINK creates link; BCAST/DELIVERY send | FOUND | Legacy LINK delivery simulation ids fail closed. |
| announcement delivery | BCAST design docs and `BroadcastClassification` | PH1.BCAST | PARTIAL | General broadcast framework exists; no separate announcement runtime path found. |
| protected/private delivery boundary | Access blueprint steps, simulation executor access enforcement, BCAST safety flags | PH1.X + Access/Governance + Authority/Simulation + BCAST/DELIVERY | PARTIAL | Confirmed for invite link and reminders; broader content privacy handshake is design-only. |
| Desktop rendering | Desktop Swift System Activity/Needs Attention surfaces | Desktop render only | PARTIAL | No external send authority found; read-only claims present. |
| iPhone rendering | iPhone `System Activity`, `Needs Attention`, broadcast/dead-letter visibility | iPhone render only | PARTIAL | No local delivery authority found. |
| Adapter transport/provider route | Adapter health/attention/reminder dispatch tests | Adapter transport/read-only runtime surface | PARTIAL | No live external provider route found; adapter must remain non-owner. |

## 4. Current Broadcast / Delivery / Reminder Lifecycles

### A. Broadcast Lifecycle

| Stage | Owner | Symbols / Files | Inputs | Outputs | State Changes | Audit / Evidence | Gaps |
|---|---|---|---|---|---|---|---|
| create draft | PH1.BCAST | `BcastDraftCreateRequest`, `run_draft_create`, `append_bcast_recipient_lifecycle_event` | tenant, sender, audience spec, classification, `content_payload_ref`, idempotency key | `broadcast_id`, `DraftCreated` | broadcast current row set to `DraftCreated` | BCAST lifecycle ledger row with request/response | Audience resolution and PH1.WRITE handoff are not fully wired. |
| select audience/recipients | Design/blueprint | `audience_spec`, `BCAST_AUDIENCE_RESOLVE` in ECM | audience selector or recipient contact | recipient ids | Not separately implemented in current BCAST runtime | Design docs only | PARTIAL. |
| validate permission | PH1.ACCESS via Selene OS | blueprints `ACCESS_GATE_DECIDE_ROW`, simulation executor tests | requester/tenant/action | access decision | No BCAST state if denied | Access tests and dispatch gates | BCAST runtime itself does not own access. |
| prepare message | PH1.X/PH1.WRITE future, BCAST payload ref current | `content_payload_ref`, blueprint message fields | subject/body/classification | payload ref | Not in BCAST state except ref | Ref carried in draft request | PH1.WRITE boundary not directly evidenced. |
| approve/confirm | PH1.X | blueprints `PH1X_CONFIRM_COMMIT_ROW` | confirmation fields | confirmation state | No BCAST state until confirmed | Blueprint evidence | Runtime confirmation proof is outside BCAST. |
| deliver request | PH1.BCAST | `BcastDeliverCommitRequest`, `run_deliver` | broadcast id, recipient id, delivery method, region, plan ref, simulation context | `delivery_request_ref`, recipient state `Waiting` or `Followup` | recipient current state set | BCAST ledger/current | External provider send is not here. |
| hand to delivery | Selene OS | `run_broadcast_deliver_with_delivery` | BCAST delivery request | DELIVERY send request/response | Delivery ledger appended if emitted | `SimulationDispatchOutcome::BroadcastDeliverySend` | Bridge is OS-owned. |
| acknowledge | PH1.BCAST | `BcastAckCommitRequest`, `run_ack` | ack status | `Concluded` | recipient state becomes `Concluded` | BCAST ledger/current | Ack semantics are minimal enums in contract. |
| defer/reminder set | PH1.BCAST + PH1.REM | `BcastDeferCommitRequest`, `run_broadcast_mhp_defer_with_reminder` | defer time, handoff flag | BCAST `ReminderSet`, REM schedule | BCAST state + REM rows | both responses returned | REM timing only. |
| reminder fired resume | PH1.REM timing then PH1.BCAST | `BCAST_REMINDER_FIRED_COMMIT`, `run_broadcast_mhp_mark_reminder_fired` | reminder ref | `ReminderFired`, follow-up decision | BCAST state transition | BCAST ledger + policy gate outcome | Voice/text follow-up delivery is a decision, not provider send proof. |
| escalate | PH1.BCAST | `BcastEscalateCommitRequest`, `run_escalate` | escalation reason | `sender_notice_ref`, state `Followup` | recipient state `Followup` | BCAST ledger/current | Sender notification delivery proof is not broadly evidenced. |
| expire/cancel | PH1.BCAST | `BcastExpireCommitRequest`, `BcastCancelCommitRequest` | sender, reason | terminal state | broadcast and recipients set `Expired` or `Canceled` | BCAST ledger/current | OK. |

### B. Delivery Lifecycle

| Stage | Owner | Symbols / Files | Inputs | Outputs | State Changes | Audit / Evidence | Gaps |
|---|---|---|---|---|---|---|---|
| receive delivery request | Selene OS to PH1.DELIVERY | `run_broadcast_deliver_with_delivery`, `Ph1DeliveryRequest::send_commit_v1` | message id, recipient, channel, payload ref, provider ref, simulation context | PH1.DELIVERY request | none yet | request object | OK. |
| resolve channel/provider | PH1.DELIVERY | `default_provider_ref_for_channel`, `binding_for_channel` | `DeliveryChannel` | provider binding | none | provider refs/KMS handles | Live providers NOT_FOUND. |
| validate recipient/contact | PH1.DELIVERY bounded string validation | `DeliverySendRequest.recipient` | recipient string | accept/refuse | none | contract validation | Consent/contact ownership underdefined. |
| send | PH1.DELIVERY | `run(Send)`, `append_delivery_attempt_event` | send request + simulation context | `delivery_attempt_id`, `delivery_proof_ref`, status | delivery ledger/current row | PH1.F append-only event | Deterministic stub, not live provider send. |
| retry/fail | PH1.DELIVERY + PH1.REM/BCAST | `provider_fail` test path, REM retry schedule | failed provider status/offline state | fail/refuse or retry schedule | REM retry/follow-up if applicable | reason codes | Provider retry/failover implementation PARTIAL. |
| status | PH1.DELIVERY | `DeliveryStatusRequest`, `DeliveryStatusResult` | attempt id/provider message ref | normalized status | no current PH1.F append for status | runtime response | Persistence of status polling not clearly wired. |
| cancel | PH1.DELIVERY | `DeliveryCancelRequest`, `DeliveryCancelResult` | attempt/provider/simulation context | canceled bool | no current PH1.F append for cancel | runtime response | Persistence of cancel not clearly wired. |
| clicked/opened | PH1.LINK/clients future | not found in DELIVERY runtime | open/click callback | status | unknown | none | NOT_FOUND for generic click/open delivery tracking outside link activation. |
| audit | PH1.F/PH1.J future | delivery ledger rows | request/response/proof | ledger/current | append-only | PH1.F row | PH1.J event emission not fully evidenced. |

### C. Reminder Lifecycle

| Stage | Owner | Symbols / Files | Inputs | Outputs | State Changes | Audit / Evidence | Gaps |
|---|---|---|---|---|---|---|---|
| create reminder | PH1.REM | `ReminderScheduleCommitRequest`, `ph1rem_schedule` | tenant/user/device, reminder type, desired time, timezone, priority, recurrence, channels | `reminder_id`, occurrence id, `Scheduled` | reminder row + occurrences | PH1.F rows | Requires identity/device records. |
| schedule recurrence | PH1.REM | `parse_reminder_recurrence_plan`, `reminder_occurrence_times` | recurrence rule | occurrence rows | occurrence rows generated | storage tests | Bounded daily/weekly support evidenced. |
| update reminder | PH1.REM | `ReminderUpdateCommitRequest`, `ph1rem_update` | fields | updated row | updates reminder and rebuilds occurrences when needed | storage tests | OK. |
| cancel reminder | PH1.REM | `ReminderCancelCommitRequest`, `ph1rem_cancel` | reminder id | `Canceled` | row and occurrences canceled | tests | OK. |
| snooze | PH1.REM | `ReminderSnoozeCommitRequest`, `ph1rem_snooze` | occurrence/snooze ms | `Snoozed` | occurrence snooze until | tests | OK. |
| trigger/firing pre/due | PH1.REM | `DeliverPreCommit`, `DeliverDueCommit`, `ph1rem_deliver_pre`, `ph1rem_deliver_due` | occurrence/delivery channel/attempt id | delivery status/proof | delivery attempt appended; state may become `FollowupPending` or retry | tests | Actual live delivery channel adapter not found. |
| retry/follow-up | PH1.REM | `ReminderFollowupScheduleCommitRequest`, `ReminderDeliveryRetryScheduleCommitRequest` | delay/retry time | followup/retry time | occurrence fields set | tests | OK for timing. |
| escalate | PH1.REM | `ReminderEscalateCommitRequest`, `ph1rem_escalate` | from/to channel, attempt id | delivered/escalation level | delivery attempt + follow-up pending | tests | Requires allowed channel and prior attempt. |
| complete/fail | PH1.REM | `MarkCompletedCommit`, `MarkFailedCommit` | ack source/failure reason | terminal state | reminder/occurrence terminal | contract/runtime | OK. |
| BCAST handoff | PH1.BCAST + PH1.REM + Selene OS | `run_broadcast_mhp_defer_with_reminder`, `run_broadcast_mhp_mark_reminder_fired` | BCAST defer/reminder fired | REM schedule, BCAST resume | both owner states updated | tests | OK. |

### D. Link Invite Delivery Lifecycle

| Stage | Owner | Symbols / Files | Inputs | Outputs | State Changes | Audit / Evidence | Gaps |
|---|---|---|---|---|---|---|---|
| create link | PH1.LINK | `execute_link`, `LINK_INVITE` blueprint | invite fields | link token/ref/url | link state | PH1.LINK extraction document | Not rewritten here. |
| ask delivery | PH1.X / blueprint | `LINK_DELIVER_INVITE`, confirmation step | requester, recipient, channel, link url | confirmation state | none | blueprint | OK. |
| create broadcast draft | PH1.BCAST | simulation executor `CreateInviteLink` delivery branch | link token ref as payload | broadcast id | BCAST draft row | PH1.F ledger | OK. |
| commit delivery request | PH1.BCAST | `BCAST_DELIVER_COMMIT` | recipient, method, link delivery plan | delivery request ref | recipient `Waiting`/`Followup` | BCAST ledger | OK. |
| provider send | PH1.DELIVERY | `DELIVERY_SEND_COMMIT` | channel/payload/provider | proof/status | delivery ledger/current | PH1.F ledger | Deterministic provider only. |
| link activation | PH1.LINK | PH1.LINK docs/runtime | receiver opens link | activation/onboarding handoff | link state | PH1.LINK extraction | Outside BCAST/DELIVERY. |
| onboarding | PH1.ONB | PH1.ONB docs | activated context | onboarding flow | onboarding state | PH1.ONB | Outside BCAST/DELIVERY. |

## 5. Data Model / Contracts / Packets

### Request Structs

| Struct / Equivalent | Owner | Path | Status | Notes |
|---|---|---|---|---|
| `Ph1BcastRequest` | PH1.BCAST | `crates/selene_kernel_contracts/src/ph1bcast.rs` | FOUND | Envelope with schema, correlation, turn, now, simulation id/type, and `BcastRequest`. |
| `BcastDraftCreateRequest` | PH1.BCAST | same | FOUND | tenant, sender, audience, classification, content payload ref, prompt dedupe, idempotency. |
| `BcastDeliverCommitRequest` | PH1.BCAST | same | FOUND | delivery method, region, app unavailable proof, delivery plan ref, simulation context. |
| `BcastDeferCommitRequest` | PH1.BCAST | same | FOUND | retry time and `handoff_to_reminder`. |
| `BcastReminderFiredCommitRequest` | PH1.BCAST | same | FOUND | resumes BCAST lifecycle after REM fires. |
| `BcastAckCommitRequest` | PH1.BCAST | same | FOUND | recipient ack status. |
| `BcastEscalateCommitRequest` | PH1.BCAST | same | FOUND | escalation reason. |
| `BcastExpireCommitRequest` | PH1.BCAST | same | FOUND | broadcast expiration. |
| `BcastCancelCommitRequest` | PH1.BCAST | same | FOUND | broadcast cancellation. |
| `Ph1DeliveryRequest` | PH1.DELIVERY | `crates/selene_kernel_contracts/src/ph1delivery.rs` | FOUND | Envelope for send/status/cancel/health. |
| `DeliverySendRequest` | PH1.DELIVERY | same | FOUND | tenant, message id, recipient, channel, payload ref, provider ref, simulation context, idempotency. |
| `DeliveryStatusRequest` | PH1.DELIVERY | same | FOUND | attempt id/provider refs. |
| `DeliveryCancelRequest` | PH1.DELIVERY | same | FOUND | attempt/provider/simulation context/idempotency. |
| `DeliveryProviderHealthCheckRequest` | PH1.DELIVERY | same | FOUND | provider and region hint. |
| `Ph1RemRequest` | PH1.REM | `crates/selene_kernel_contracts/src/ph1rem.rs` | FOUND | Envelope for reminder requests. |
| `ReminderScheduleCommitRequest` | PH1.REM | same | FOUND | schedule inputs including timezone, priority, recurrence, channels. |
| `ReminderUpdateCommitRequest` | PH1.REM | same | FOUND | update fields. |
| `ReminderCancelCommitRequest` | PH1.REM | same | FOUND | cancel reason/idempotency. |
| `ReminderSnoozeCommitRequest` | PH1.REM | same | FOUND | bounded snooze duration. |
| `ReminderFollowupScheduleCommitRequest` | PH1.REM | same | FOUND | bounded follow-up delay. |
| `ReminderDeliveryRetryScheduleCommitRequest` | PH1.REM | same | FOUND | retry time. |
| `ReminderDeliverPreCommitRequest` / `ReminderDeliverDueCommitRequest` | PH1.REM | same | FOUND | delivery attempt proof for pre/due reminder. |
| `ReminderEscalateCommitRequest` | PH1.REM | same | FOUND | channel escalation. |
| `ReminderMarkCompletedCommitRequest` / `ReminderMarkFailedCommitRequest` | PH1.REM | same | FOUND | terminal occurrence changes. |

### Response Structs / Outcomes

| Struct / Equivalent | Owner | Status | Notes |
|---|---|---|---|
| `Ph1BcastOk` / `Ph1BcastRefuse` / `Ph1BcastResponse` | PH1.BCAST | FOUND | OK requires `no_authority_grant=true` and `simulation_gated=true`. |
| `BcastOutcome` variants | PH1.BCAST | FOUND | Draft, deliver, defer, reminder fired, ack, escalate, expire, cancel. |
| `Ph1DeliveryOk` / `Ph1DeliveryRefuse` / `Ph1DeliveryResponse` | PH1.DELIVERY | FOUND | OK includes capability, simulation id, reason code, outcome, simulation gate passed. |
| `DeliveryOutcome` variants | PH1.DELIVERY | FOUND | Send, status, cancel, provider health check. |
| `Ph1RemOk` / `Ph1RemRefuse` / `Ph1RemResponse` | PH1.REM | FOUND | OK can carry reminder id, occurrence id, state, delivery status, proof, escalation level. |

### Records / Current Projections

| Record | Owner | Path | Status | Notes |
|---|---|---|---|---|
| `BcastRecipientLifecycleLedgerRow` | PH1.F / PH1.BCAST evidence | `crates/selene_storage/src/ph1f.rs` | FOUND | Append-only BCAST lifecycle row with request/response, capability, reason code, idempotency. |
| `BcastBroadcastCurrentRecord` | PH1.F / PH1.BCAST current | same | FOUND | Rebuildable broadcast state projection. |
| `BcastRecipientLifecycleCurrentRecord` | PH1.F / PH1.BCAST current | same | FOUND | Rebuildable per-recipient state projection. |
| `DeliveryAttemptLedgerRow` | PH1.F / PH1.DELIVERY evidence | same | FOUND | Append-only send-attempt row with status/proof/request/response. |
| `DeliveryAttemptCurrentRecord` | PH1.F / PH1.DELIVERY current | same | FOUND | Current delivery attempt projection. |
| `BcastPolicyLedgerRow` / `BcastPolicyCurrentRecord` | PH1.F / BCAST policy | same | FOUND | Unified policy values for wait/urgent/max/reminder default. |
| `BcastWaitPolicyLedgerRow` / current | PH1.F / BCAST policy | same | FOUND | Separate wait policy projection. |
| `BcastUrgentFollowupPolicyLedgerRow` / current | PH1.F / BCAST policy | same | FOUND | Separate urgent follow-up policy projection. |
| `ReminderRecord` | PH1.F / PH1.REM | same | FOUND | Current reminder row. |
| `ReminderOccurrenceRecord` | PH1.F / PH1.REM | same | FOUND | Per-occurrence row. |
| `ReminderDeliveryAttemptRecord` | PH1.F / PH1.REM | same | FOUND | Append-only reminder delivery attempt proof row. |
| SQL comms/reminder tables | database migrations | `crates/selene_storage/migrations/*.sql` | NOT_FOUND | DESIGN_GAP for durable SQL schema if production persistence requires it. |

### Enums / Status States

| Enum | Owner | Values | Status |
|---|---|---|---|
| `BroadcastClassification` | PH1.BCAST | `Simple`, `Priority`, `Emergency` | FOUND |
| `BcastDeliveryMethod` | PH1.BCAST | `SeleneApp`, `Sms`, `Whatsapp`, `Wechat`, `Email` | FOUND |
| `BcastRecipientRegion` | PH1.BCAST | `Global`, `China` | FOUND |
| `BcastRecipientState` | PH1.BCAST | `DraftCreated`, `Waiting`, `Followup`, `ReminderSet`, `ReminderFired`, `Deferred`, `Concluded`, `Canceled`, `Expired` | FOUND |
| `BcastAckStatus` | PH1.BCAST | `Received`, `ActionConfirmed`, `Declined` | FOUND |
| `DeliveryChannel` | PH1.DELIVERY | `Sms`, `Email`, `Whatsapp`, `Wechat`, `AppPush` | FOUND |
| `DeliveryStatus` | PH1.DELIVERY | `Sent`, `Pending`, `Failed`, `Canceled`, `NotSupported` | FOUND |
| `DeliveryProviderHealthState` | PH1.DELIVERY | `Healthy`, `Degraded`, `Unavailable` | FOUND |
| `ReminderType` | PH1.REM | `Task`, `Meeting`, `Timer`, `Medical`, `Custom`, `BcastMhpFollowup` | FOUND |
| `ReminderState` | PH1.REM | `Scheduled`, `Snoozed`, `FollowupPending`, `Canceled`, `Completed`, `Failed` | FOUND |
| `ReminderDeliveryStatus` | PH1.REM | `Delivered`, `DeferredQuietHours`, `RetryScheduled`, `Failed` | FOUND |
| `ReminderChannel` | PH1.REM | `Voice`, `Push`, `Text`, `Email`, `PhoneApp` | FOUND |

### Error Types / Reason Codes

| Area | Reason Codes / Errors | Status | Notes |
|---|---|---|---|
| BCAST runtime | `BCAST_FAIL_SCHEMA_INVALID`, `BCAST_FAIL_NOT_FOUND`, `BCAST_FAIL_STATE_TRANSITION_INVALID`, `BCAST_FAIL_SIMULATION_CONTEXT_MISSING`, `BCAST_FAIL_WAITING_WINDOW_NOT_ELAPSED`, `BCAST_FAIL_FALLBACK_POLICY`, `BCAST_FAIL_INTERNAL` | FOUND | Runtime reason-code constants. |
| DELIVERY runtime | `DELIVERY_SIMULATION_CONTEXT_MISSING`, `DELIVERY_CHANNEL_UNAVAILABLE`, `DELIVERY_PROVIDER_SEND_FAILED`, `DELIVERY_ATTEMPT_NOT_FOUND`, `DELIVERY_PROVIDER_STATUS_UNAVAILABLE`, `DELIVERY_CANCEL_NOT_SUPPORTED`, `DELIVERY_PROVIDER_CANCEL_FAILED`, `DELIVERY_PROVIDER_HEALTH_UNAVAILABLE` | FOUND | Runtime reason codes implied by code/tests. |
| REM storage runtime | `REM_FAIL_TIME_AMBIGUOUS_NEEDS_CONFIRM`, `REM_FAIL_SCOPE_VIOLATION`, `REM_FAIL_STATE_TRANSITION_INVALID`, `REM_FAIL_POLICY_BLOCKED` | FOUND | Current PH1.F implementation; docs list additional reason codes not all found in runtime constants. |

## 6. Product Function Types

| Function Type | Evidence Found | Current Behavior | Owner | Security Implications | Missing Design |
|---|---|---|---|---|---|
| direct message | `MESSAGE_COMPOSE_AND_SEND` blueprint | Clarify/confirm, access gate, BCAST draft/deliver, DELIVERY send | PH1.X, Access, PH1.BCAST, PH1.DELIVERY | External send side effect; confirmation and access required | PH1.WRITE draft boundary not directly wired in BCAST runtime. |
| broadcast announcement | BCAST docs/contracts | Draft/deliver lifecycle supports broadcast semantics | PH1.BCAST | Audience/classification can be high risk | Multi-recipient runtime resolution PARTIAL. |
| invite/link delivery | `LINK_DELIVER_INVITE`, simulation executor link delivery branch | PH1.LINK creates, BCAST drafts/delivers, DELIVERY sends | PH1.LINK, PH1.BCAST, PH1.DELIVERY | Link delivery requires Access+Simulation; raw link does not grant authority | Live provider send proof missing. |
| reminder notification | `REMINDER_MANAGE`, PH1.REM runtime | Schedule/fire/update/cancel/attempt proof | PH1.REM | Timing side effect; must remain scoped to tenant/user | Live notification delivery provider missing. |
| scheduled reminder | PH1.REM contract/runtime/tests | Deterministic scheduling and recurrence | PH1.REM | Identity/device ownership checked | SQL migration missing. |
| follow-up notification | BCAST.MHP + REM handoff | BCAST state -> REM timing -> BCAST resume | PH1.BCAST + PH1.REM | Follow-up voice requires speaker continuity in OS bridge | Live UX proof deferred. |
| system notification | iPhone/Desktop System Activity surfaces | Read-only operational visibility | Clients render only | Client must not mutate/send | Runtime-to-client feed shape not fully extracted. |
| onboarding notification | `ONB_REQUIREMENT_BACKFILL` | ONB campaign progress after BCAST/REM handoff | PH1.ONB + PH1.BCAST + PH1.REM | Delivery cannot onboard/grant access | PARTIAL runtime coverage. |
| emotional/outreach draft delivery | Emotional docs not implementation evidence | Drafting may exist in future through PH1.WRITE/Delivery | PH1.WRITE then BCAST/DELIVERY | Sending is side effect, not emotion-owned | NOT_FOUND in current BCAST runtime. |
| customer/supplier/staff notification | Blueprint/general broadcast semantics | Possible by audience/recipient refs | PH1.BCAST | Access/tenant scope needed | Specific owner resolver missing. |
| SMS | DELIVERY provider binding | Deterministic proof ref only | PH1.DELIVERY | SMS setup pre-send gate in docs | No Twilio/live SMS proof. |
| email | DELIVERY provider binding | Deterministic proof ref only | PH1.DELIVERY | External send side effect | No SMTP/SendGrid proof. |
| in-app/AppPush | BCAST `SeleneApp` -> DELIVERY `AppPush` | Primary delivery path | PH1.BCAST + PH1.DELIVERY + client render | Client render cannot equal proof of business success | Live push provider proof missing. |
| push notification | AppPush binding and client surfaces | Deterministic provider stub | PH1.DELIVERY | Device/client must not own authority | APNS/Firebase proof NOT_FOUND. |
| QR/copy-link handoff | Simulation catalog legacy docs mention QR/CopyLink | Not found as current BCAST/DELIVERY runtime channel | PH1.LINK/client future | Copy/QR can leak link | NOT_FOUND in current runtime. |
| delivery status/retry/failover | DELIVERY status/cancel, REM retry, BCAST fallback | Deterministic status and retry pieces exist | PH1.DELIVERY/PH1.REM/PH1.BCAST | Must be idempotent and audited | End-to-end provider retry/failover PARTIAL. |

## 7. Access / Identity / Tenant / Workspace / Permission Interaction

Sending a message is a side effect.

Sending to a customer, supplier, staff member, or company member may require permission.

Public-safe personal/friend link delivery may be lower risk, but still needs confirmation and contact/channel validation.

Broadcast to many recipients is higher risk.

External delivery must be audited.

Delivery must not bypass Access/Governance.

PH1.BCAST / PH1.DELIVERY must not create links; PH1.LINK creates links.

PH1.BCAST / PH1.DELIVERY must not onboard users; PH1.ONB does onboarding.

PH1.BCAST / PH1.DELIVERY must not grant access.

Repo enforcement evidence:

- `docs/DB_WIRING/PH1_OS.md` explicitly locks outbound delivery ownership: PH1.LINK owns token/draft only, PH1.BCAST owns lifecycle, PH1.DELIVERY owns provider-attempt proof only, PH1.REM owns timing only.
- `docs/BLUEPRINTS/LINK_DELIVER_INVITE.md` requires confirmation and `ACCESS_GATE_DECIDE_ROW` before BCAST/DELIVERY delivery.
- `docs/BLUEPRINTS/MESSAGE_COMPOSE_AND_SEND.md` requires a confirmation step and `ACCESS_GATE_DECIDE_ROW` before send.
- `docs/DB_WIRING/PH1_ACCESS_001_PH2_ACCESS_002.md` includes SMS setup gate behavior and says Selene OS orchestrates escalation delivery; Access does not call BCAST directly.
- `crates/selene_os/src/simulation_executor.rs` enforces governed link delivery and rejects legacy LINK delivery simulation ids through `is_legacy_link_delivery_simulation_id`.
- PH1.BCAST OK responses validate `no_authority_grant=true`.
- PH1.E has a negative test proving a hostile `send_email` tool request is rejected as forbidden.

Repo gaps:

- Contact consent/opt-in is not fully evidenced as a runtime owner.
- Audience resolution and private/confidential privacy handshake are documented in ECM/design, but not fully implemented as BCAST runtime request variants.
- Tenant/workspace binding is strongest in link/reminder dispatch and storage ids; broad broadcast audience tenant scoping still needs future proof.

## 8. PH1.WRITE / OpenAI / Message Wording Interaction

Current repo evidence:

- BCAST runtime uses `content_payload_ref`. It does not store or generate final message body text.
- `MESSAGE_COMPOSE_AND_SEND` blueprint gathers `subject_topic`, `body_content`, classification, receipt mode, and confirmation before BCAST/DELIVERY.
- PH1.X prompt helpers ask channel/contact questions for link/message flows.
- No direct PH1.WRITE -> PH1.BCAST runtime handoff was found in current code.
- No OpenAI provider-assisted message wording path was found inside PH1.BCAST/PH1.DELIVERY/PH1.REM runtime.

Correct future rule:

PH1.WRITE should own final user-facing/message wording. OpenAI/GPT-5.5 may propose drafts through PH1.D. Broadcast/Delivery must not invent final content by itself unless repo truth proves a bounded template owner.

Current risks:

- `BCAST_WRITING_OWNER_RISK`: PARTIAL. BCAST currently carries payload refs rather than text, which is good, but final PH1.WRITE content boundary is not wired in the extracted BCAST runtime evidence.
- `DELIVERY_TEMPLATE_RISK`: PARTIAL. PH1.DELIVERY carries payload refs and provider refs, not templates. Future template handling needs an owner.
- `HARDCODED_MESSAGE_RISK`: LOW in BCAST/DELIVERY runtime, because message bodies are refs. Some client Swift contains user-facing explanatory copy, but not delivery message bodies.

## 9. Channel Strategy and Provider Surfaces

| Channel / Provider Surface | Evidence | Channel Status | Provider-Off Behavior | Fake-Provider Behavior | Live-Provider Risk | Cost/Audit Evidence | Missing Policy |
|---|---|---|---|---|---|---|---|
| Selene App / AppPush | `BcastDeliveryMethod::SeleneApp`, `DeliveryChannel::AppPush`, `provider:app_push/default` | PARTIAL | No live provider path found | Deterministic runtime returns proof ref | Live APNS/push NOT_FOUND | KMS handle ref in deterministic proof | Provider governance/cost evidence missing. |
| SMS | `DeliveryChannel::Sms`, `provider:sms/default`, `kms://delivery/sms/default` | PARTIAL | Disabled binding can refuse | Runtime deterministic proof/refuse | Twilio/live SMS NOT_FOUND | PH1.F delivery ledger stores proof ref | Provider-off/fake-provider certification missing. |
| Email | `DeliveryChannel::Email`, `provider:email/default` | PARTIAL | Disabled binding can refuse | Runtime deterministic proof/refuse | SMTP/SendGrid NOT_FOUND | Proof ref includes KMS handle | Provider policy missing. |
| WhatsApp | `DeliveryChannel::Whatsapp`, fallback order docs/tests | PARTIAL | Disabled binding can refuse | Runtime deterministic proof/refuse | Live WhatsApp provider NOT_FOUND | Proof ref includes KMS handle | Regional/compliance policy missing. |
| WeChat | `DeliveryChannel::Wechat`, China fallback path, health degraded without China region hint | PARTIAL | Disabled binding can refuse | Runtime deterministic health/status | Live WeChat provider NOT_FOUND | Proof ref includes KMS handle | China-specific provider policy missing. |
| Phone/voice reminder | `ReminderChannel::PhoneApp`, BCAST follow-up voice default | PARTIAL | Not provider-backed in current evidence | OS decision emits follow-up refs | Live voice follow-up proof deferred | Reminder proof refs exist for attempts | PH1.K/PH1.TTS/live audible proof needed later. |

No Twilio, SendGrid, SMTP, APNS, Firebase push, WhatsApp Business, or live WeChat provider integration was found in the inspected repo surfaces. Current delivery provider behavior is deterministic and local to runtime logic.

## 10. Desktop / iPhone / Adapter Boundaries

Desktop behavior:

- Desktop Swift surfaces include `System Activity`, `Needs Attention`, pending/failed queue visibility, and onboarding action submissions.
- Desktop code contains explicit non-authoritative copy for onboarding/photo/sender verification surfaces.
- No evidence was found that Desktop owns external delivery, provider routing, broadcast lifecycle decisions, link creation, or reminder timing.

iPhone behavior:

- iPhone `SessionShellView.swift` includes read-only operational cards such as `broadcast_waiting_followup_reminder_state`, `dead_letter_or_failed_delivery`, and `broadcast_followup_requires_human_action`.
- The iPhone shell copy explicitly says these surfaces do not resend, repair transport, mutate queues, or complete work locally.
- No evidence was found that iPhone owns external delivery, provider routing, broadcast lifecycle decisions, link creation, or reminder timing.

Adapter behavior:

- Adapter code includes operational health/dead-letter visibility and tests for reminder dispatch through simulation/runtime.
- No current evidence shows the Adapter owning SMS/email/WhatsApp/WeChat provider sends.
- No current evidence shows Adapter granting delivery permission or mutating BCAST/DELIVERY/REM outside runtime orchestration.

Wrong-owner risks:

- `DESKTOP_DELIVERY_AUTHORITY_RISK`: NOT_FOUND in current evidence. Keep guarded because UI has pending/failed visibility.
- `IPHONE_DELIVERY_AUTHORITY_RISK`: NOT_FOUND in current evidence. Keep guarded because UI has action-copy surfaces.
- `ADAPTER_DELIVERY_AUTHORITY_RISK`: PARTIAL. Adapter has operational queues and reminder dispatch tests; future reconciliation must prove it remains transport/runtime entry only.

## 11. Security / Consent / Privacy Model

| Security Area | Repo Evidence | Status | Notes |
|---|---|---|---|
| recipient/contact validation | bounded string/ids in contracts, PH1.X helpers for channel/contact examples | PARTIAL | Robust contact book/consent owner not found. |
| consent/opt-in | blueprints require confirmation before send | PARTIAL | Recipient opt-in/unsubscribe not found. |
| tenant/workspace restrictions | tenant ids in BCAST/DELIVERY/REM requests and storage | PARTIAL | Tenant ids are carried; broad workspace binding not fully evidenced for delivery. |
| role/permission checks | Access blueprint steps and simulation executor gates | FOUND | BCAST/DELIVERY do not grant authority. |
| external send confirmation | `LINK_DELIVER_INVITE`, `MESSAGE_COMPOSE_AND_SEND`, `REMINDER_MANAGE` blueprints | FOUND | Confirmation is blueprint-level, not BCAST-owned. |
| message preview/approval | message blueprint confirmation fields | PARTIAL | No PH1.WRITE preview runtime in BCAST found. |
| audit | PH1.F append-only ledgers/proof refs | PARTIAL | PH1.J event emission not fully found for every path. |
| idempotency | BCAST, DELIVERY, REM idempotency indexes | FOUND | Strong repo evidence in runtime/storage/tests. |
| retry/fail behavior | BCAST fallback/defer/escalate, REM retry/follow-up, DELIVERY provider fail reason | PARTIAL | End-to-end live retry/failover missing. |
| rate limits/caps | policy docs mention rate limits; no current BCAST runtime cap found | DESIGN_GAP | Needed for broad broadcasts. |
| sensitive data restrictions | DB docs require privacy handshake for private/confidential; BCAST payload ref model | PARTIAL | Privacy handshake runtime not implemented as request variant. |
| link privacy | PH1.LINK extraction; link payload ref used for delivery | FOUND | Link creation/activation outside this document. |
| private/protected data in message bodies | Access and PH1.WRITE future boundary | DESIGN_GAP | Need PH1.WRITE/content classification enforcement proof. |
| unsubscribe/stop handling | not found in BCAST/DELIVERY runtime | NOT_FOUND | Needed for external messaging compliance. |

## 12. Broadcast / Delivery / Reminder State Machines

### BCAST State Machine

RECONSTRUCTED_FROM_REPO_EVIDENCE:

```text
DraftCreated
  -> Waiting                 via BCAST_DELIVER_COMMIT, non-urgent or app/default delivery
  -> Followup                via BCAST_DELIVER_COMMIT for Emergency urgent-immediate

Waiting
  -> Deferred                via BCAST_DEFER_COMMIT with handoff_to_reminder=false
  -> Followup                via BCAST_ESCALATE_COMMIT after wait window
  -> Concluded               via BCAST_ACK_COMMIT
  -> Canceled                via BCAST_CANCEL_COMMIT
  -> Expired                 via BCAST_EXPIRE_COMMIT

Followup
  -> ReminderSet             via BCAST_DEFER_COMMIT with handoff_to_reminder=true
  -> Deferred                via BCAST_DEFER_COMMIT with handoff_to_reminder=false
  -> Concluded               via BCAST_ACK_COMMIT
  -> Canceled                via BCAST_CANCEL_COMMIT
  -> Expired                 via BCAST_EXPIRE_COMMIT

ReminderSet
  -> ReminderFired           via BCAST_REMINDER_FIRED_COMMIT
  -> Canceled                via BCAST_CANCEL_COMMIT
  -> Expired                 via BCAST_EXPIRE_COMMIT

ReminderFired
  -> ReminderSet             via BCAST_DEFER_COMMIT with handoff_to_reminder=true
  -> Deferred                via BCAST_DEFER_COMMIT with handoff_to_reminder=false
  -> Concluded               via BCAST_ACK_COMMIT
  -> Canceled                via BCAST_CANCEL_COMMIT
  -> Expired                 via BCAST_EXPIRE_COMMIT

Deferred
  -> Followup                via BCAST_ESCALATE_COMMIT
  -> Concluded               via BCAST_ACK_COMMIT
  -> Canceled                via BCAST_CANCEL_COMMIT
  -> Expired                 via BCAST_EXPIRE_COMMIT

Concluded | Canceled | Expired
  -> terminal for new deliver attempts
```

`SENT` is used in BCAST.MHP docs as a conceptual phone-first lifecycle label. Current `BcastRecipientState` uses `Waiting`/`Followup` after deliver commit rather than a separate `Sent` enum.

### DELIVERY State Machine

RECONSTRUCTED_FROM_REPO_EVIDENCE:

```text
Send request
  -> Sent              provider binding available, simulation_context present, provider success
  -> Failed            provider failure/refuse recorded in PH1.F append path
  -> Pending           status normalization can return pending
  -> Canceled          cancel result can return canceled
  -> NotSupported      enum exists, not strongly evidenced in runtime send path
```

Current storage append path is specifically for send attempts. Status/cancel runtime contracts exist, but persistent status/cancel event projection is PARTIAL.

### REM State Machine

RECONSTRUCTED_FROM_REPO_EVIDENCE:

```text
Scheduled
  -> Snoozed           via REMINDER_SNOOZE_COMMIT
  -> FollowupPending   via REMINDER_DELIVER_DUE_COMMIT delivered or REMINDER_FOLLOWUP_SCHEDULE_COMMIT
  -> Canceled          via REMINDER_CANCEL_COMMIT
  -> Completed         via REMINDER_MARK_COMPLETED_COMMIT
  -> Failed            via REMINDER_MARK_FAILED_COMMIT

Snoozed
  -> RetryScheduled delivery status / retry_time if due arrives before snooze_until
  -> FollowupPending after due delivery when allowed
  -> Canceled | Completed | Failed

FollowupPending
  -> delivered/escalated attempts may append proof
  -> Canceled | Completed | Failed

Canceled | Completed | Failed
  -> terminal for mutation/delivery
```

Reminder delivery statuses are `Delivered`, `DeferredQuietHours`, `RetryScheduled`, and `Failed`. The current runtime uses retry scheduling for offline/early/snoozed due delivery paths and appends attempt proof references.

## 13. Error Handling and Reason Codes

| Error / Reason Area | Current Evidence | Status |
|---|---|---|
| invalid recipient | BCAST unknown broadcast/recipient and REM scope checks | PARTIAL |
| missing contact | PH1.X/blueprint required field clarification | PARTIAL |
| unsupported channel | DELIVERY channel binding and provider mismatch checks | PARTIAL |
| channel provider disabled | DELIVERY binding availability can refuse | FOUND |
| provider failure | `provider_fail` payload path returns `DELIVERY_PROVIDER_SEND_FAILED` | FOUND |
| delivery timeout | not found | NOT_FOUND |
| retry exhausted | REM mark failed, retry scheduling; max attempts in BCAST policy | PARTIAL |
| permission denied | Access gate docs/tests | FOUND |
| tenant mismatch | REM scope violation, link tenant mismatch tests | FOUND for REM/link; PARTIAL for generic BCAST delivery |
| message too long | not found in BCAST/DELIVERY runtime | NOT_FOUND |
| template missing | not found | NOT_FOUND |
| link missing | PH1.LINK extraction covers link; BCAST uses token ref payload | PARTIAL |
| link expired before send | PH1.LINK extraction covers; BCAST not owner | PARTIAL |
| reminder expired | not a current REM state; canceled/completed/failed terminal exist | DESIGN_GAP |
| duplicate send/idempotency conflict | BCAST/DELIVERY idempotency indexes return existing event/proof | FOUND |
| protected/private content blocked | Access/blueprints/docs; privacy handshake runtime missing | PARTIAL |
| SMS provider unavailable | DELIVERY binding unavailable can refuse | PARTIAL |
| email provider unavailable | DELIVERY binding unavailable can refuse | PARTIAL |

## 14. Audit / Provenance / Evidence

| Question | Repo Answer | Status |
|---|---|---|
| Is broadcast creation audited? | PH1.F append-only `BcastRecipientLifecycleLedgerRow` stores draft request/response, reason code, idempotency. | FOUND |
| Is delivery send audited? | PH1.F append-only `DeliveryAttemptLedgerRow` stores send request/response, status, proof ref, idempotency. | FOUND |
| Is failed delivery audited? | Refused send can be appended as `DeliveryStatus::Failed`; simulation executor has best-effort send-link delivery audit on failure. | PARTIAL |
| Is retry audited? | REM retry schedule and BCAST defer/escalate events are stored; live provider retry audit is PARTIAL. | PARTIAL |
| Is reminder scheduling audited? | PH1.F stores reminder rows; docs say audit events emitted. | PARTIAL |
| Is reminder firing audited? | Reminder delivery attempts are append-only; BCAST reminder-fired lifecycle event is append-only. | FOUND |
| Are recipient/channel refs recorded? | BCAST recipient id and delivery method; DELIVERY recipient/channel; REM channel. | FOUND |
| Are link delivery refs recorded? | Simulation executor passes `link_token_ref`, BCAST content payload ref, delivery proof ref. | FOUND |
| Are provider response refs recorded? | DELIVERY current record stores `provider_message_ref` for send OK. | FOUND |
| Are old compatibility route events logged? | Legacy LINK delivery simulations fail closed; specific event logging beyond reason code is PARTIAL. | PARTIAL |
| Are PH1.J audit events emitted for all paths? | Docs require PH1.J; code evidence in inspected paths is incomplete. | AUDIT_GAP |

## 15. Current Tests / Smokes / Acceptance

| Test / Smoke | Path | What It Proves | What It Does Not Prove | Status |
|---|---|---|---|---|
| `at_bcast_contract_01_simulation_id_must_match_variant` | `crates/selene_kernel_contracts/src/ph1bcast.rs` | BCAST request simulation id must match variant | Live workflow | FOUND |
| `at_bcast_contract_02_deliver_requires_simulation_context` | same | Delivery commit requires simulation context | Access gate | FOUND |
| `at_bcast_contract_02a_fallback_requires_app_unavailable_flag` | same | Fallback requires app unavailable flag | Live app availability proof | FOUND |
| `at_bcast_contract_02b_fallback_requires_app_unavailable_proof_ref` | same | Fallback requires proof ref | Proof source validation | FOUND |
| `at_bcast_contract_03_ok_requires_safety_flags_true` | same | OK response cannot grant authority or skip simulation gate | Full protected lane | FOUND |
| `at_bcast_01`..`at_bcast_10` | `crates/selene_engines/src/ph1bcast.rs` | BCAST runtime idempotency, state transitions, fallback order, wait policy, urgent behavior, mutex poison fail-closed | Live delivery | FOUND |
| `at_bcast_wiring_01`..`03` | `crates/selene_os/src/ph1bcast.rs` | OS wiring validation, disabled path, drift fail-closed | Full OS dispatch | FOUND |
| `at_delivery_contract_01`..`03` | `crates/selene_kernel_contracts/src/ph1delivery.rs` | Delivery constructor, simulation mismatch fail-closed, capability mismatch | Live providers | FOUND |
| `at_delivery_01`..`04` | `crates/selene_engines/src/ph1delivery.rs` | SMS deterministic send, provider failure, WeChat cancel unsupported, unknown provider health fail-closed | Twilio/SendGrid/real provider | FOUND |
| `at_delivery_wiring_01`..`05` | `crates/selene_os/src/ph1delivery.rs` | OS delivery wiring validation/disabled/drift errors | Live provider | FOUND |
| `at_rem_contract_01`..`02` | `crates/selene_kernel_contracts/src/ph1rem.rs` | Reminder sim id match and delivery attempt id requirement | Runtime scheduling quality | FOUND |
| `at_rem_01`..`02` | `crates/selene_os/src/ph1rem.rs` | Reminder schedule idempotency and ambiguous time refusal | Full reminder lifecycle | FOUND |
| `at_rem_db_01`..`11` | `crates/selene_os/src/ph1rem.rs` | Round trip rows, append-only attempts, recurrence, update/cancel, due fires once, retry no double send, escalation policy, snooze, missed due follow-up | Live notification delivery | FOUND |
| `at_f_11`..`at_f_18` | `crates/selene_storage/src/ph1f.rs` | BCAST/DELIVERY append-only replay and BCAST policy projections | SQL migrations | FOUND |
| `at_sim_exec_01b`..`01g` | `crates/selene_os/src/simulation_executor.rs` | BCAST->REM handoff, urgent/non-urgent follow-up, app-thread reply forward, fallback order, BCAST->DELIVERY bridge | Live client audible/visible proof | FOUND |
| `at_sim_exec_14`..`18` | `crates/selene_os/src/simulation_executor.rs` | Link access deny/escalate/tenant mismatch/idempotency and legacy LINK delivery fail-closed | Live invite send | FOUND |
| `check_delivery_ownership_boundaries.sh` | `scripts/check_delivery_ownership_boundaries.sh` | Static ownership guardrails for LINK/BCAST/DELIVERY/REM | Runtime behavior by itself | FOUND |
| Adapter reminder tests | `crates/selene_adapter/src/lib.rs` | Calendar/reminder confirm dispatch and cancel/list reminder behavior | External notification provider | PARTIAL |
| Live SMS/email/WhatsApp/WeChat acceptance | Not found | Nothing | Live provider behavior | TEST_GAP |

## 16. Old Paths / Compatibility / Wrong-Owner Risks

| Path / Symbol | Current Status | Correct Canonical Owner | Retirement Condition | Active-Caller Check Needed |
|---|---|---|---|---|
| `LINK_INVITE_SEND_COMMIT` | `LEGACY_DO_NOT_WIRE` in simulation catalog; fail-closed in `execute_link` | `LINK_DELIVER_INVITE` via PH1.BCAST + PH1.DELIVERY | Keep blocked unless formally retired from catalog | Yes |
| `LINK_INVITE_RESEND_COMMIT` | `LEGACY_DO_NOT_WIRE` | same | same | Yes |
| `LINK_DELIVERY_FAILURE_HANDLING_COMMIT` | `LEGACY_DO_NOT_WIRE` | same | same | Yes |
| PH1.LINK delivery ownership | Blocked by docs/script/runtime | PH1.LINK creates/activates links only | Never reintroduce delivery into PH1.LINK | Yes |
| BCAST direct PH1.DELIVERY call | Guard script requires absence in BCAST runtime/wiring | Selene OS bridge | Maintain guard | Yes |
| REM direct PH1.BCAST/DELIVERY call | Guard script requires absence in REM runtime | Selene OS bridge | Maintain guard | Yes |
| Adapter delivery shortcuts | No external send route found; operational queues exist | Adapter transport/read-only visibility | Prove no provider send or business decision | Yes |
| Client send shortcuts | Desktop/iPhone render/action-copy surfaces exist | Clients render/submit bounded requests only | Prove no local external send authority | Yes |
| hardcoded messages | Swift explanatory copy and PH1.X prompts exist; BCAST carries payload refs | PH1.WRITE/future template owner | Wire final wording boundary | Yes |
| PH1.E `send_email` | Negative test rejects forbidden tool | PH1.DELIVERY for external send | Keep PH1.E read-only | Yes |
| BCAST audience/privacy design-only capabilities | ECM lists but runtime does not implement request variants | PH1.BCAST + Access + PH1.WRITE | Implement only through explicit build | Yes |

## 17. Full Functionality Master List

| Functionality | Description | Evidence | Owner | Status | Future Action |
|---|---|---|---|---|---|
| create broadcast draft | Create canonical broadcast/delivery envelope with classification and payload ref | `BCAST_CREATE_DRAFT`, `run_draft_create` | PH1.BCAST | FOUND | Keep; integrate PH1.WRITE payload handoff. |
| resolve recipients | Turn audience/contact into recipient ids | ECM/blueprints | PH1.BCAST + PH1.X + Access | PARTIAL | Build recipient/audience resolver proof. |
| create delivery request | Commit BCAST delivery request and produce `delivery_request_ref` | `BCAST_DELIVER_COMMIT` | PH1.BCAST | FOUND | Keep OS bridge only. |
| send SMS | Deterministic provider send/proof ref for SMS | `DeliveryChannel::Sms` | PH1.DELIVERY | PARTIAL | Add provider-off/fake/live SMS proof later. |
| send email | Deterministic provider send/proof ref for Email | `DeliveryChannel::Email` | PH1.DELIVERY | PARTIAL | Add provider proof later. |
| send WhatsApp | Deterministic provider send/proof ref for WhatsApp | `DeliveryChannel::Whatsapp` | PH1.DELIVERY | PARTIAL | Add regional/provider proof later. |
| send WeChat | Deterministic provider send/proof ref for WeChat | `DeliveryChannel::Wechat` | PH1.DELIVERY | PARTIAL | Add regional/provider proof later. |
| send AppPush | Deterministic provider send/proof ref for AppPush | `DeliveryChannel::AppPush` | PH1.DELIVERY | PARTIAL | Add push provider/client proof later. |
| delivery status | Normalize provider status | `DeliveryStatusRequest` | PH1.DELIVERY | PARTIAL | Persist status polling events. |
| cancel delivery | Cancel provider attempt where supported | `DeliveryCancelRequest` | PH1.DELIVERY | PARTIAL | Persist cancel events and provider behavior. |
| provider health | Check provider health state/latency bucket | `DeliveryProviderHealthCheckRequest` | PH1.DELIVERY | FOUND | Add real provider policy/circuit evidence later. |
| schedule reminder | Create reminder and occurrences | `REMINDER_SCHEDULE_COMMIT` | PH1.REM | FOUND | Add SQL persistence if needed. |
| update reminder | Update reminder and rebuild occurrence schedule | `REMINDER_UPDATE_COMMIT` | PH1.REM | FOUND | Keep. |
| cancel reminder | Cancel reminder and occurrences | `REMINDER_CANCEL_COMMIT` | PH1.REM | FOUND | Keep. |
| snooze reminder | Temporarily defer occurrence | `REMINDER_SNOOZE_COMMIT` | PH1.REM | FOUND | Keep. |
| schedule follow-up | Set follow-up pending time | `REMINDER_FOLLOWUP_SCHEDULE_COMMIT` | PH1.REM | FOUND | Keep. |
| schedule retry | Set retry time | `REMINDER_DELIVERY_RETRY_SCHEDULE_COMMIT` | PH1.REM | FOUND | Keep. |
| deliver pre/due reminder | Append reminder delivery attempt/proof | `REMINDER_DELIVER_PRE_COMMIT`, `REMINDER_DELIVER_DUE_COMMIT` | PH1.REM timing/proof | FOUND / PARTIAL | Clarify provider handoff relationship. |
| escalate reminder | Escalate channel if policy allows | `REMINDER_ESCALATE_COMMIT` | PH1.REM | FOUND | Add policy owner proof. |
| mark completed/failed | Terminal occurrence transitions | `REMINDER_MARK_COMPLETED_COMMIT`, `REMINDER_MARK_FAILED_COMMIT` | PH1.REM | FOUND | Keep. |
| deliver invite link | Send generated link via BCAST/DELIVERY | `LINK_DELIVER_INVITE`, simulation executor `LinkDelivered` | PH1.LINK + PH1.BCAST + PH1.DELIVERY | FOUND | Keep split; add live proof later. |
| track click/open/delivered status | Link activation and delivery current rows | PH1.LINK extraction, DELIVERY current | PARTIAL | Add clicked/open callback proof if needed. |
| app-thread reply auto-forward | Conclude BCAST and send AppPush response | `run_broadcast_mhp_app_thread_reply_conclude` | PH1.BCAST + PH1.DELIVERY | FOUND | Ensure generic naming and no hardcoded persona path later. |
| BCAST fallback order | App-first fallback SMS/WhatsApp/WeChat/Email only when app unavailable | BCAST runtime/tests | PH1.BCAST | FOUND | Keep guard. |
| BCAST policy updates | Non-urgent wait and urgent follow-up policy records | PH1.F policy rows, simulation executor tests | PH1.BCAST policy | FOUND | Map to governance owner during reconciliation. |

## 18. Comparison To Master Architecture

Global Request Decision Lattice:

Current blueprints route user intent through PH1.X confirmation/access/simulation steps before BCAST/DELIVERY/REM side effects. This matches the rule that outbound communication is not a raw transcript action.

PH1.D Proposal Gateway:

No direct PH1.D provider proposal path was found inside BCAST/DELIVERY/REM. Future message drafting, channel-choice explanation, and delivery status wording should use PH1.D only as a proposal gateway through PH1.WRITE/PH1.X.

PH1.N Meaning Unravelling:

Message, reminder, and link delivery blueprints still reference older `PH1.NLP` wording. Future reconciliation should map this to PH1.N repo-equivalent ownership without creating a duplicate NLP/router.

PH1.WRITE Human Presentation:

BCAST/DELIVERY currently carry refs and status, not final human wording. PH1.WRITE must become the explicit final-content and user-facing explanation owner for messages, confirmations, failures, and delivery summaries.

PH1.LINK Link Journey:

Current repo truth agrees with PH1.LINK journey design: PH1.LINK creates/governs link lifecycle, while BCAST/DELIVERY sends and REM/ONB handle timing/onboarding handoffs. Legacy LINK delivery paths are explicitly blocked.

Identity + Access + Authority Spine:

Access gates are blueprint/OS prerequisites. BCAST/DELIVERY/REM do not grant access or authority. Protected/private delivery still needs stronger content-level proof.

Broadcast / Delivery / Reminder / Messaging Stack:

Repo truth already has a meaningful foundation, but it is split across contracts, engine runtime, OS bridges, PH1.F storage, docs, and tests. Durable provider and SQL migration proof remain gaps.

Onboarding / Invite / Link / Enrollment Stack:

ONB backfill and invite delivery both rely on BCAST/DELIVERY/REM. ONB remains owner of campaign/target progress and onboarding state; delivery does not onboard.

Selene Emotional Intelligence / Outreach Drafts:

No current BCAST/DELIVERY/REM implementation for emotional outreach send was found. Future outreach must draft through PH1.WRITE and send only through delivery gates.

Desktop/iPhone Render-Only Boundary:

Current clients expose read-only operational visibility and bounded runtime request surfaces. They do not appear to own external sending.

Adapter Transport-Only Boundary:

Adapter has reminder/operational visibility tests and dead-letter posture, but no external provider ownership found. Future proof must keep provider/business decisions out of Adapter.

Old Compatibility Path Retirement:

Legacy LINK delivery simulations are already guarded as `LEGACY_DO_NOT_WIRE`. Retirement should happen only after active-caller checks and catalog/doc reconciliation.

## 19. Gaps / Missing Pieces

| Gap | Evidence | Risk | Recommended Future Action | Priority |
|---|---|---|---|---|
| missing live SMS provider proof | provider refs only; no Twilio/live code | Sends may be overclaimed | Provider-off/fake/live SMS proof pack | High |
| missing live email provider proof | provider refs only; no SMTP/SendGrid | Sends may be overclaimed | Email provider governance slice | High |
| missing app push provider proof | AppPush deterministic binding only | Client notification not proven | Push provider + Desktop/iPhone render proof | High |
| missing delivery status persistence for status/cancel | PH1.F append path is send-specific | Status/cancel truth may be transient | Extend PH1.F or repo-equivalent ledgers | Medium |
| missing contact consent/unsubscribe | no owner found | Compliance and privacy risk | Consent/contact resolver activation | High |
| missing audience resolver runtime | ECM design only | Wrong recipients, broad broadcast risk | Audience resolver build slice | High |
| missing privacy handshake runtime | ECM design only | Private/confidential content leakage risk | BCAST privacy handshake implementation/proof | High |
| missing PH1.WRITE content boundary | BCAST uses refs but no explicit write handoff | BCAST could become content owner later | PH1.WRITE message payload contract | High |
| missing SQL migrations for comms/reminders | migration search found no BCAST/DELIVERY/REM tables | Persistence gap for production | PH1.F durable schema plan | Medium |
| missing PH1.J audit emission proof | PH1.F ledgers exist; docs require PH1.J | Audit/proof may be incomplete | Audit evidence pack | Medium |
| missing live retry/failover | deterministic retry state only | Delivery reliability not proven | Delivery retry/failure proof | Medium |
| unclear provider policy/cost counters | no provider governance/cost path found | Hidden spend/provider drift risk | Provider Governance integration | Medium |
| client authority risk must stay watched | clients show action-copy and visibility | UI could grow local send controls | Desktop/iPhone render-only proof | Medium |
| adapter provider risk must stay watched | adapter has operational queues | Adapter could absorb delivery logic | Adapter transport-only proof | Medium |
| no JD live acceptance | tests are code-level | Product behavior not accepted visibly | JD live broadcast/delivery/reminder acceptance | Medium |

## 20. Recommended Future Build Slices

1. PH1.BCAST / PH1.DELIVERY / PH1.REM Repo-Truth Activation Pack
2. Broadcast Contract / State Machine Normalization
3. Recipient / Audience Resolver Activation
4. PH1.WRITE Message Draft Boundary
5. Delivery Channel Matrix / SMS-First Proof
6. Provider-Off / Fake Delivery Provider Proof
7. SMS Delivery Handoff Proof
8. Link Delivery Handoff Proof
9. Reminder Schedule / Fire Proof
10. Delivery Status / Retry / Failure Proof
11. Audit Evidence Pack
12. Desktop/iPhone Render-Only Delivery Proof
13. Adapter Transport-Only Delivery Proof
14. Old Compatibility Retirement Ledger
15. JD Live Broadcast/Delivery/Reminder Acceptance Pack

Additional repo-truth-derived slices:

16. BCAST Privacy Handshake / Private Content Guard
17. Contact Consent / Unsubscribe / Channel Permission Map
18. PH1.F Durable SQL Migration Plan for comms/reminder tables
19. BCAST.MHP Follow-Up Voice/Text Proof With PH1.K/PH1.TTS Boundaries
20. BCAST Policy Governance Owner Reconciliation

## 21. What Codex Must Not Do

Codex must not invent broadcast behavior.

Codex must not create duplicate broadcast/delivery/reminder engines.

Codex must not send from PH1.LINK.

Codex must not create links from PH1.BCAST/DELIVERY.

Codex must not onboard users from delivery.

Codex must not grant access from delivery.

Codex must not let Desktop/iPhone send externally.

Codex must not let Adapter own provider/business delivery logic.

Codex must not bypass PH1.WRITE for final message wording where content is user-facing.

Codex must not send protected/private content without access/authority.

Codex must not delete old paths before proof.

Codex must not implement from this extraction document alone.

Codex must not claim live provider behavior from deterministic provider refs.

Codex must not treat visible client notification/render success as business delivery proof.

Codex must not treat reminder timing as message authority.

Codex must not let BCAST state transitions grant access, identity, onboarding completion, or protected execution.

## 22. Final Extracted Architecture Sentence

PH1.BCAST / PH1.DELIVERY / PH1.REM form Selene’s outbound communication boundary: they may draft, schedule, queue, send, retry, and prove delivery where repo truth supports it, but message wording, link creation, onboarding, access, authority, and protected execution must remain owned by their canonical Selene engines.
