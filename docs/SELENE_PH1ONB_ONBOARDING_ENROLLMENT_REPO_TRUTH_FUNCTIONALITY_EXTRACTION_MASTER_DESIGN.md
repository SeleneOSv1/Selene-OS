# Selene PH1.ONB Onboarding + Enrollment — Repo-Truth Functionality Extraction Master Design

DOCUMENT STATUS:
REPO_TRUTH_EXTRACTION
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

## 0. Authority and Scope

AGENTS.md controls execution.

This is a docs-only repo-truth extraction.

No runtime code was changed.

This document does not authorize implementation.

The document reconstructs current onboarding/enrollment design and functionality from repo evidence.

Future implementation/refactor/retirement requires explicit build instruction, approved file scope, tests, backend evidence, JD approval, and where visible JD live proof.

This extraction is not a new architecture invention. It names current behavior, current owners, current tests, and current gaps. Where current repo truth is incomplete, this document marks the area as `NOT_FOUND`, `PARTIAL`, `UNKNOWN`, `REPO_TRUTH_NEEDED`, `DESIGN_GAP`, `TEST_GAP`, `OWNER_GAP`, or `AUDIT_GAP`.

## 1. Executive Summary

PH1.ONB is Selene's current governed onboarding-session execution boundary.

Based on repo truth, PH1.ONB currently supports invited onboarding that starts only after PH1.LINK has activated an invite/open link and produced app-open context. PH1.ONB creates or reuses one deterministic onboarding session per activated link token, pins onboarding schema context, tracks missing required fields, records terms state, handles schema-required employee photo/sender verification gates, records primary device proof, coordinates voice/wake/persona setup prerequisites, creates a per-user access instance through the PH2 access storage path, completes onboarding, consumes the activated link, and records requirement-backfill campaign progress.

PH1.ONB is separate from PH1.LINK. PH1.LINK owns link creation, token/signature validation, expiry/revocation/blocking, app-open activation, and token lifecycle. PH1.ONB consumes activated link context and does not own token validation.

PH1.ONB is separate from Access/Governance. Current storage wiring lets ONB access-provisioning create a PH2 access instance after deterministic onboarding prerequisites pass, but the correct future boundary remains that Access/Governance owns access policy, role templates, permission scope, and any approval/escalation decisions. This current repo behavior is `PARTIAL` because ONB calls the access storage path directly from the onboarding continuation flow.

PH1.ONB is separate from Voice ID. Current runtime orchestration invokes PH1.VOICE.ID enrollment simulations and requires a locked voice enrollment plus sync receipt before completion. Voice ID remains evidence and enrollment truth; it does not grant authority.

PH1.ONB is separate from Broadcast/Delivery/Reminder. Repo docs define onboarding backfill notification handoff through PH1.BCAST and PH1.REM, while PH1.ONB records campaign/target progress only after those handoffs. PH1.ONB does not send messages directly in the extracted design.

Current onboarding types are active for invited onboarding, employee/person invite paths, schema-driven employee requirement gates, business prerequisite blueprints, and requirement backfill. Self onboarding, customer onboarding, supplier onboarding, contractor onboarding, friend/personal connection onboarding, and full tenant/workspace onboarding are `PARTIAL` or `NOT_FOUND` as concrete PH1.ONB runtime flows, even though invitee types and future blueprint language mention some categories.

Desktop and iPhone surfaces are currently client/bridge surfaces. They parse app-open/invite-open route context, call canonical Adapter endpoints, render bounded onboarding outcomes, and in some Desktop paths submit bounded continuation actions to Adapter. They must not become onboarding authority owners.

## 2. Repo Evidence Inventory

| Evidence Area | File / Path | Symbols / Routes / Tables | Status | Notes |
|---|---|---|---|---|
| PH1.ONB kernel contract | `crates/selene_kernel_contracts/src/ph1onb.rs` | `PH1ONB_CONTRACT_VERSION`, `ONB_SESSION_START_DRAFT`, `ONB_TERMS_ACCEPT_COMMIT`, `ONB_EMPLOYEE_PHOTO_CAPTURE_SEND_COMMIT`, `ONB_EMPLOYEE_SENDER_VERIFY_COMMIT`, `ONB_PRIMARY_DEVICE_CONFIRM_COMMIT`, `ONB_ACCESS_INSTANCE_CREATE_COMMIT`, `ONB_COMPLETE_COMMIT`, `ONB_REQUIREMENT_BACKFILL_*`, `OnboardingStatus`, `OnboardingNextStep`, `Ph1OnbRequest`, `Ph1OnbResponse`, `OnbRequest`, result structs | FOUND | Canonical contract for current ONB request/result variants and status enums. |
| Dedicated engines crate PH1.ONB file | `crates/selene_engines/src/ph1onb.rs` | None | NOT_FOUND | Current repo does not expose a standalone PH1.ONB engine crate file. Runtime owner is OS orchestration plus PH1.F storage. |
| PH1.ONB OS runtime | `crates/selene_os/src/ph1onb.rs` | `Ph1OnbOrchRuntime`, `run`, `start_session_from_link_activation`, `run_voice_enrollment_live_sequence`, `run_position_live_sequence`, `audit_transition`, `reason_codes::*` | FOUND | Main ONB runtime dispatch and orchestration helper. Emits PH1.J audit transitions. |
| App ingress bridge | `crates/selene_os/src/app_ingress.rs` | `AppInviteLinkOpenRequest`, `AppInviteLinkOpenOutcome`, `AppOnboardingContinueAction`, `AppOnboardingContinueRequest`, `AppOnboardingContinueOutcome`, `run_invite_link_open_and_start_onboarding`, `run_onboarding_continue` | FOUND | Canonical app/open route bridge for invite-click start and onboarding continuation. |
| Simulation executor integration | `crates/selene_os/src/simulation_executor.rs` | PH1.ONB import/use, active simulation checks, invite creation, link delivery, test setup for ONB and app ingress | FOUND | Executor supports PH1.LINK, PH1.REM, PH1.ONB and enforces simulation catalog activation for invite/onboarding paths. |
| PH1.LINK contract/runtime | `crates/selene_kernel_contracts/src/ph1link.rs`, `crates/selene_os/src/ph1link.rs` | `LinkActivationResult`, `LinkStatus`, `LINK_INVITE_OPEN_ACTIVATE_COMMIT`, invite draft/open/activate flows | FOUND | PH1.ONB starts only after PH1.LINK activation; token validation remains PH1.LINK-owned. |
| Storage current state | `crates/selene_storage/src/ph1f.rs` | `OnboardingSessionRecord`, `OnbAskMissingOutcome`, `ph1onb_session_start_draft`, `ph1onb_ask_missing_field_turn`, `ph1onb_*_commit`, backfill functions | FOUND | Primary current-state runtime truth for ONB sessions, missing-field state, receipts, idempotency, access instance link, completion. |
| Typed storage repo | `crates/selene_storage/src/repo.rs` | `Ph1OnbRepo`, `ph1onb_session_start_draft_row`, `ph1onb_session_row`, `ph1onb_terms_accept_commit_row`, `ph1onb_complete_commit_row`, backfill row methods | FOUND | Typed repository facade for ONB DB wiring tests and storage row calls. |
| SQL link/onboarding draft tables | `crates/selene_storage/migrations/0012_ph1link_onboarding_draft_tables.sql` | `onboarding_drafts`, `onboarding_link_tokens`, `onboarding_draft_write_dedupe` | FOUND | SQL migration is LINK/onboarding-draft focused. It notes runtime wiring remains in-memory for current MVP slices. |
| SQL position/backfill tables | `crates/selene_storage/migrations/0014_position_requirements_schema_and_backfill_tables.sql` | `position_requirements_schema_ledger`, `position_requirements_schema_current`, `onboarding_requirement_backfill_campaigns`, `onboarding_requirement_backfill_targets` | FOUND | SQL support for position requirements schema and ONB requirement backfill campaign/target state. |
| SQL voice enrollment tables | `crates/selene_storage/migrations/0008_ph1vid_voice_enrollment_tables.sql` | `voice_enrollment_sessions`, `voice_enrollment_samples`, `voice_profiles`, `voice_profile_bindings` | FOUND | Voice enrollment has onboarding-session linkage. Physical FK is deferred; storage wiring enforces it in current slice. |
| SQL wake enrollment tables | `crates/selene_storage/migrations/0011_ph1w_wake_tables.sql` | `wake_enrollment_sessions`, `wake_enrollment_samples` | FOUND | Wake enrollment optionally links to onboarding session; ONB completion checks wake receipt by platform policy. |
| ONB DB wiring doc | `docs/DB_WIRING/PH1_ONB.md` | `status: PASS`, `onboarding_sessions`, idempotency indexes, ownership/reads/writes/tests | FOUND | Strong design-lock evidence for current PH1.ONB scope and acceptance proof targets. |
| ONB ECM doc | `docs/ECM/PH1_ONB.md` | `PH1ONB_SESSION_START_DRAFT_ROW`, `PH1ONB_*_ROW`, failure modes, audit notes | FOUND | Current engine/capability matrix for ONB runtime capabilities. |
| Invited onboarding blueprint | `docs/BLUEPRINTS/ONB_INVITED.md` | `ONB_INVITED`, S01-S16 flow, PH1.LINK handoff, PH1.VOICE.ID, PH1.W, PH1.ONB completion | FOUND | Blueprint-level process flow. Some symbols are blueprint-only or compatibility names. |
| Business setup blueprint | `docs/BLUEPRINTS/ONB_BIZ_SETUP.md` | `ONB_BIZ_START_DRAFT`, `ONB_BIZ_VALIDATE_COMPANY_COMMIT`, PH1.POSITION company upsert/read | PARTIAL | Business onboarding prerequisite is blueprint-intent; runtime evidence points to PH1.POSITION owning company rows. |
| Schema management blueprint | `docs/BLUEPRINTS/ONB_SCHEMA_MANAGE.md` | Position schema create/update/activate; optional ONB backfill | FOUND | Establishes PH1.POSITION as schema owner and PH1.ONB as executor/backfill progress owner. |
| Requirement backfill blueprint | `docs/BLUEPRINTS/ONB_REQUIREMENT_BACKFILL.md` | Backfill campaign steps, PH1.BCAST/PH1.REM handoff, ONB notify/complete commits | FOUND | Confirms ONB records progress; BCAST/REM own outbound delivery/timing. |
| Simulation catalog | `docs/08_SIMULATION_CATALOG.md` | ONB simulation entries, LINK entries, VOICE/W wake enrollment entries, backfill entries | FOUND | Lists ONB simulations and legacy/blueprint `ONB_DRAFT_UPDATE_COMMIT` entry. |
| Blueprint registry | `docs/09_BLUEPRINT_REGISTRY.md` | `ONB_INVITED`, `ONB_BIZ_SETUP`, `ONB_SCHEMA_MANAGE`, `ONB_REQUIREMENT_BACKFILL` | FOUND | Registry declares ONB blueprints active. |
| Adapter library | `crates/selene_adapter/src/lib.rs` | `InviteLinkOpenAdapterRequest`, `InviteLinkOpenAdapterResponse`, `OnboardingContinueAdapterRequest`, `OnboardingContinueAdapterResponse`, `run_onboarding_continue`, `parse_onboarding_continue_action` | FOUND | Adapter translates HTTP/API payloads into app ingress requests; should remain transport/translation. |
| HTTP adapter routes | `crates/selene_adapter/src/bin/http_adapter.rs` | `/v1/invite/click`, `/v1/onboarding/continue`, ingress security checks | FOUND | HTTP routes enforce bearer/request/timestamp/nonce security before runtime call. |
| iPhone client | `apple/iphone/SeleneIPhone/SessionShellView.swift` | `onboardingEntryActive`, `inviteLike`, `openLike`, onboarding outcome/continue rows | FOUND | iPhone shows read-only onboarding entry and outcome state; no local authority found in inspected evidence. |
| Mac Desktop client | `apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`, `apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift` | `/v1/invite/click`, `/v1/onboarding/continue`, onboarding prompt/state cards, bounded action builders | FOUND | Desktop calls canonical adapter endpoints and repeatedly states non-authoritative posture; some continuation actions originate from client UI but route to runtime. |
| Storage tests | `crates/selene_storage/tests/ph1_onb/db_wiring.rs` | `at_onb_db_*`, `runc_onb_db_*`, `rund_onb_db_*` | FOUND | Proves tenant isolation, app-open handoff, idempotency, schema pinning, sender verification blocks, wake/voice receipts, backfill, ask-missing state, platform receipts. |
| OS/app ingress tests | `crates/selene_os/src/app_ingress.rs` | `run1_invite_link_click_*`, onboarding continue tests around missing fields, terms, device, voice, wake, persona, access, complete | FOUND | In-source tests cover invite click start and continuation sequencing. |
| Adapter tests | `crates/selene_adapter/src/lib.rs`, `crates/selene_adapter/src/bin/http_adapter.rs` | onboarding continue parser tests, HTTP security tests, iPhone invite onboarding E2E | FOUND | Proves adapter route translation/security and bounded onboarding continuation. |
| Readiness scripts | `scripts/selene_design_readiness_audit.sh` | onboarding draft contract parity checks | FOUND | Script-level guard for onboarding/link contract parity. |

## 3. Current Owner Map

| Responsibility | Current Owner / File | Correct Future Owner Hypothesis | Status | Notes |
|---|---|---|---|---|
| onboarding session start | `Ph1OnbOrchRuntime` + `Ph1fStore::ph1onb_session_start_draft` | PH1.ONB | FOUND | Requires activated link plus app-open context. |
| invited onboarding session start | `AppServerIngressRuntime::run_invite_link_open_and_start_onboarding` | PH1.LINK activates; PH1.ONB starts session | FOUND | Ingress checks active `LINK_INVITE_OPEN_ACTIVATE_COMMIT` and `ONB_SESSION_START_DRAFT`. |
| onboarding from link activation | `Ph1OnbOrchRuntime::start_session_from_link_activation`, app ingress bridge | PH1.LINK -> PH1.ONB handoff | FOUND | ONB consumes activation result; does not validate signature/expiry itself. |
| onboarding field collection | `ph1onb_ask_missing_field_turn`, `AppOnboardingContinueAction::AskMissingSubmit` | PH1.ONB with PH1.N/PH1.X/PH1.WRITE guidance later | FOUND | Current runtime updates link draft through `ph1link_invite_draft_update_commit`. |
| missing required fields | `OnboardingSessionRecord.missing_fields`, `active_missing_field`, `asked_missing_fields` | PH1.ONB | FOUND | Repo enforces one active missing-field state in session row. |
| onboarding step progression | `AppOnboardingContinueNextStep` and app ingress continuation match arms | PH1.ONB/App ingress orchestration | FOUND | Steps include ask-missing, platform setup, terms, device, voice, wake, persona lock, access, complete, ready, blocked. |
| onboarding completion | `ONB_COMPLETE_COMMIT`, `ph1onb_complete_commit` | PH1.ONB, with Access/Voice/Wake prerequisites | FOUND | Consumes activated/opened link and requires access instance plus voice/wake receipts. |
| onboarding cancellation | Terms declined -> blocked state; blueprint mentions stop on never | PH1.ONB + PH1.X/PH1.WRITE future | PARTIAL | No broad cancellation API found beyond terms decline/blocking and blueprint text. |
| onboarding recovery | App ingress idempotency/replay, deterministic reuse by token | PH1.ONB + PH1.LINK + clients | PARTIAL | Recovery/state resume exists through deterministic rows; full user-facing recovery design not found. |
| onboarding requirement backfill | `ph1onb_requirement_backfill_*`, `ONB_REQUIREMENT_BACKFILL` blueprint | PH1.ONB for progress; BCAST/REM for delivery/timing | FOUND | `NewHiresOnly` must not enter backfill. |
| onboarding status/progress | `OnboardingStatus`, `AppOnboardingContinueNextStep`, response structs | PH1.ONB / PH1.WRITE presentation | FOUND | Runtime returns status/next step; final natural explanation boundary is underdefined. |
| employee onboarding | `InviteeType::Employee`, position schema pinning, employee verification gates | PH1.ONB + PH1.POSITION + Access | FOUND | Strongest current onboarding type. |
| customer onboarding | `InviteeType::Customer` in link draft SQL enum; no full ONB customer runtime flow found | PH1.ONB + customer owner | PARTIAL | Invitee type exists; specific customer onboarding steps/gates are not evidenced. |
| supplier onboarding | Search did not find concrete PH1.ONB supplier runtime type | Supplier onboarding owner + PH1.ONB future | NOT_FOUND | Mark as design gap if needed later. |
| friend/personal onboarding | `InviteeType::Friend`, `FamilyMember`, `Associate` in LINK draft enum | PH1.ONB or personal connection owner | PARTIAL | Token/draft type exists; dedicated ONB journey is not proved. |
| tenant/workspace onboarding | Tenant scope is enforced; workspace onboarding not found as runtime path | Access/Governance + Workspace owner + PH1.ONB | PARTIAL | Tenant field exists; workspace-specific ONB not found. |
| role/access template handoff | `ph1onb_access_instance_create_commit` calls `ph2access_upsert_instance_commit` | Access/Governance owns policy/templates; ONB coordinates after gates | PARTIAL | Current path writes access instance from ONB continuation; future reconciliation should prove policy owner. |
| voice enrollment handoff | `run_voice_enrollment_live_sequence`, PH1.VOICE.ID storage/migrations | PH1.VOICE.ID owns voice enrollment truth | FOUND | ONB coordinates sequence and requires receipt. |
| device enrollment handoff | `ONB_PRIMARY_DEVICE_CONFIRM_COMMIT`, platform setup receipts | Device/Presence owner + PH1.ONB | PARTIAL | Primary device proof and platform receipts exist; full device enrollment owner is broader than ONB. |
| identity proof/photo/document checks | employee photo proof refs; sender verification | PH1.ONB for refs; document/artifact owner for raw proof | PARTIAL | No full KYC/photo-ID document proof owner found in ONB. |
| onboarding reminders/follow-ups | `ONB_REQUIREMENT_BACKFILL` blueprint uses PH1.BCAST and PH1.REM | PH1.BCAST / PH1.DELIVERY / PH1.REM | PARTIAL | Backfill handoff exists in docs; direct runtime notification send not ONB-owned. |
| onboarding notifications | Backfill notify commit after BCAST/REM handoff | PH1.BCAST/DELIVERY | PARTIAL | PH1.ONB records progress but does not send. |
| Desktop rendering | Desktop Swift shell and runtime bridge | Desktop render/submit bounded inputs only | FOUND | Desktop calls adapter endpoints; should not own ONB authority. |
| iPhone rendering | iPhone SessionShellView | iPhone render/open only | FOUND | Read-only onboarding entry/outcome previews. |
| Adapter transport | Adapter lib and HTTP routes | Adapter transport/validation only | FOUND | Adapter parses payloads and calls runtime; should not own ONB state. |
| audit/provenance | `Ph1OnbOrchRuntime::audit_transition`, PH1.J | PH1.J audit owner; PH1.ONB emits bounded transitions | FOUND | Audit is present for runtime transitions; some client/adapter visibility remains `PARTIAL`. |
| storage/migrations | PH1.F storage, migrations 0012/0014/0008/0011 | PH1.F storage + SQL migrations | PARTIAL | ONB current state is primarily PH1.F in-memory/current-state wiring; no standalone SQL `onboarding_sessions` migration found. |
| old compatibility paths | legacy aliases in docs, `ONB_DRAFT_UPDATE_COMMIT`, client `inviteLike/openLike/appOpenLike` parsing | Retain until active-caller proof; retire after reconciliation | PARTIAL | Must not delete in this task. |

## 4. Current Onboarding Lifecycle

### Stage 1: onboarding request or invite draft exists upstream

- owner: PH1.LINK / invite generation path.
- symbols/files: `LINK_INVITE_GENERATE_DRAFT`, `onboarding_drafts`, `InviteeType`, `crates/selene_storage/migrations/0012_ph1link_onboarding_draft_tables.sql`.
- inputs: inviter, invitee type, tenant scope, optional prefilled context.
- outputs: onboarding draft and token context.
- state changes: draft/link token records are PH1.LINK-owned.
- audit evidence: LINK audit belongs to PH1.LINK; not PH1.ONB.
- gaps/unknowns: customer/supplier/friend-specific onboarding behavior after draft is `PARTIAL`.

### Stage 2: invited link activated

- owner: PH1.LINK.
- symbols/files: `LINK_INVITE_OPEN_ACTIVATE_COMMIT`, `LinkActivationResult`, `LinkStatus::Activated`, `AppInviteLinkOpenRequest`, `/v1/invite/click`.
- inputs: token id, token signature, device fingerprint, platform, app instance id, deep link nonce.
- outputs: activation result with status, prefilled context ref, app-open fields, missing required fields.
- state changes: link transitions to `ACTIVATED` or fails/refuses/blocks under PH1.LINK rules.
- audit evidence: PH1.LINK audit/evidence outside PH1.ONB.
- gaps/unknowns: none for the current ONB handoff; PH1.LINK extraction remains the deeper source.

### Stage 3: onboarding session started

- owner: PH1.ONB.
- symbols/files: `ONB_SESSION_START_DRAFT`, `Ph1OnbRequest::session_start_draft_v1`, `ph1onb_session_start_draft`, `OnboardingSessionRecord`.
- inputs: activated token id, prefilled context ref, tenant id, device fingerprint, app platform, app instance id, deep link nonce, link opened time.
- outputs: `OnbSessionStartResult`, deterministic `onboarding_session_id`, `OnboardingStatus::DraftCreated`, `OnboardingNextStep`, pinned schema context, required verification gates.
- state changes: creates or reuses one session by activated token.
- audit evidence: `ONB_OK_SESSION_START_DRAFT` PH1.J transition from `NONE` to `DRAFT_CREATED`.
- gaps/unknowns: physical SQL table for `onboarding_sessions` is not found as a migration; current DB wiring describes PH1.F current-state runtime slice.

### Stage 4: required fields computed and asked

- owner: PH1.ONB current state plus PH1.LINK draft update for the underlying draft.
- symbols/files: `missing_fields`, `active_missing_field`, `ph1onb_ask_missing_field_turn`, `LINK_INVITE_DRAFT_UPDATE_COMMIT`, `AppOnboardingContinueAction::AskMissingSubmit`.
- inputs: onboarding session id, optional field value, idempotency key.
- outputs: `OnbAskMissingOutcome` as `Prompt`, `Updated`, or `Escalated`; remaining missing fields.
- state changes: `asked_missing_fields`, `active_missing_field`, attempts, and remaining missing fields update.
- audit evidence: storage state evidence; app ingress returns prompt state.
- gaps/unknowns: PH1.WRITE natural guidance ownership is not fully wired for prompts; current prompt question generation is deterministic/hardcoded.

### Stage 5: platform setup receipts

- owner: PH1.ONB current state; client supplies bounded receipt evidence; app ingress validates order.
- symbols/files: `AppOnboardingContinueAction::PlatformSetupReceipt`, `ph1onb_platform_setup_receipt_commit`, `remaining_platform_receipt_kinds`.
- inputs: receipt kind, receipt ref, signer, payload hash, onboarding session.
- outputs: accepted receipt kind and remaining required receipt kinds.
- state changes: receipt maps on `OnboardingSessionRecord`.
- audit evidence: storage state; tests prove round trip.
- gaps/unknowns: exact platform setup receipt policy owner is `PARTIAL`.

### Stage 6: terms accepted or declined

- owner: PH1.ONB.
- symbols/files: `ONB_TERMS_ACCEPT_COMMIT`, `TermsStatus`, `OnbTermsAcceptCommitRequest`.
- inputs: onboarding session id, terms version id, accepted flag, idempotency key.
- outputs: `TermsStatus::Accepted` or `TermsStatus::Declined`.
- state changes: status becomes `TermsAccepted` or `TermsDeclined`.
- audit evidence: `ONB_OK_TERMS_ACCEPT_COMMIT` transition.
- gaps/unknowns: final user wording for decline/blocked state is `PARTIAL`.

### Stage 7: schema-required employee photo evidence and sender verification

- owner: PH1.ONB for evidence refs and verification state; external/artifact/delivery owner for raw evidence and notification future.
- symbols/files: `ONB_EMPLOYEE_PHOTO_CAPTURE_SEND_COMMIT`, `ONB_EMPLOYEE_SENDER_VERIFY_COMMIT`, `required_verification_gates`, `PHOTO_EVIDENCE_CAPTURE`, `SENDER_CONFIRMATION`.
- inputs: photo blob ref, sender user id, decision, idempotency keys.
- outputs: photo proof ref, verification status.
- state changes: status moves through `VerificationPending`, `VerificationConfirmed`, or `VerificationRejected`.
- audit evidence: `ONB_OK_EMPLOYEE_PHOTO_COMMIT`, `ONB_OK_EMPLOYEE_SENDER_VERIFY_COMMIT`.
- gaps/unknowns: the capability name says "send", but current storage writes refs and verification state. Actual message delivery belongs outside ONB and is `PARTIAL`.

### Stage 8: primary device proof

- owner: PH1.ONB with runtime governance proof check.
- symbols/files: `ONB_PRIMARY_DEVICE_CONFIRM_COMMIT`, `ProofType`, `PrimaryDeviceConfirm`, `govern_protected_action_proof`.
- inputs: device id, proof type, proof ok, idempotency key.
- outputs: primary device confirmed boolean.
- state changes: `primary_device_device_id`, `primary_device_proof_type`, `primary_device_confirmed`, status `PrimaryDeviceConfirmed` when ok.
- audit evidence: `ONB_OK_PRIMARY_DEVICE_CONFIRM_COMMIT`.
- gaps/unknowns: broader device identity enrollment owner is `PARTIAL`.

### Stage 9: voice enrollment and optional wake enrollment

- owner: PH1.VOICE.ID and PH1.W for enrollment truth; PH1.ONB orchestrates and gates completion.
- symbols/files: `run_voice_enrollment_live_sequence`, `VOICE_ID_ENROLL_START_DRAFT`, `VOICE_ID_ENROLL_SAMPLE_COMMIT`, `VOICE_ID_ENROLL_COMPLETE_COMMIT`, `WAKE_ENROLL_*`, migrations 0008 and 0011.
- inputs: onboarding session id, device id, consent asserted, sample refs/metrics, wake enrollment actions where required.
- outputs: voice enrollment session, voice sync receipt, wake sync receipt where required.
- state changes: voice/wake tables/state, device artifact sync worker pass.
- audit evidence: voice/wake owners plus ONB completion check.
- gaps/unknowns: voice enrollment is present; full user-consent UX and cross-device policy need future proof.

### Stage 10: persona/tone setup lock

- owner: PH1.EMO / PH1.PERSONA for tone/persona proposals and snapshots; PH1.ONB records lock audit event.
- symbols/files: `AppOnboardingContinueAction::EmoPersonaLock`, `ph1onb_emo_persona_lock_commit`, `emo_persona_lock_audit_event_id`, PH1.EMO/PH1.PERSONA calls in app ingress.
- inputs: onboarding session id, tenant, primary device, voice profile, tone-only evidence.
- outputs: PH1.PERSONA audit event id and ONB lock field.
- state changes: `emo_persona_lock_audit_event_id`.
- audit evidence: referenced PH1.PERSONA audit event and ONB storage link.
- gaps/unknowns: product naming is normalized to Selene; persona lock remains a technical setup step and must not grant authority.

### Stage 11: access instance provisioning

- owner: current runtime is PH1.ONB calling PH2 access storage; future policy owner is Access/Governance.
- symbols/files: `ONB_ACCESS_INSTANCE_CREATE_COMMIT`, `ph1onb_access_instance_create_commit`, `ph2access_upsert_instance_commit`, `access_engine_instance_id`.
- inputs: onboarding session id, user id, tenant id, role id, idempotency key.
- outputs: access engine instance id.
- state changes: identity row may be inserted if absent; PH2 access instance is upserted; ONB status becomes `AccessInstanceCreated`.
- audit evidence: `ONB_OK_ACCESS_INSTANCE_CREATE_COMMIT`.
- gaps/unknowns: `OWNER_GAP`: future reconciliation must prove Access/Governance owns role/access policy and ONB is only the setup coordinator.

### Stage 12: onboarding completed

- owner: PH1.ONB with prerequisites from Access, Voice ID, Wake, and link state.
- symbols/files: `ONB_COMPLETE_COMMIT`, `ph1onb_complete_commit`, `OnboardingStatus::Complete`.
- inputs: onboarding session id, idempotency key, voice receipt ref, optional wake receipt ref.
- outputs: completion status.
- state changes: status becomes `Complete`; link becomes `Consumed` when activated/opened; receipts are stored.
- audit evidence: `ONB_OK_COMPLETE_COMMIT`; device artifact sync worker pass.
- gaps/unknowns: full final PH1.WRITE presentation and JD live visible acceptance are `DESIGN_GAP`/`TEST_GAP`.

### Stage 13: onboarding failed/expired/cancelled

- owner: partial.
- symbols/files: `TermsDeclined`, `Blocked`, link expiry/revocation/blocking in PH1.LINK, blueprint failure modes.
- inputs: decline, rejected sender verification, failed proof, invalid link, missing simulation, policy refusal.
- outputs: blocked/refused/error states.
- state changes: terms decline and verification reject are represented; broad cancel/expire ONB session states are not first-class current `OnboardingStatus` values.
- audit evidence: partial through PH1.J for runtime transitions and adapter error responses.
- gaps/unknowns: `DESIGN_GAP` for explicit ONB cancellation/recovery/expiry state model.

## 5. Data Model / Contracts / Packets

### Request structs and API equivalents

| Name | File / Path | Kind | Status | Notes |
|---|---|---|---|---|
| `OnbSessionStartDraftRequest` | `crates/selene_kernel_contracts/src/ph1onb.rs` | request struct | FOUND | Contains token, prefilled context ref, tenant, device/app-open context. |
| `OnbTermsAcceptCommitRequest` | `crates/selene_kernel_contracts/src/ph1onb.rs` | request struct | FOUND | Records terms version and accepted boolean. |
| `OnbEmployeePhotoCaptureSendCommitRequest` | `crates/selene_kernel_contracts/src/ph1onb.rs` | request struct | FOUND | Stores photo/evidence refs and sender user id. |
| `OnbEmployeeSenderVerifyCommitRequest` | `crates/selene_kernel_contracts/src/ph1onb.rs` | request struct | FOUND | Confirms or rejects sender verification. |
| `OnbPrimaryDeviceConfirmCommitRequest` | `crates/selene_kernel_contracts/src/ph1onb.rs` | request struct | FOUND | Records device id, proof type, proof ok. |
| `OnbAccessInstanceCreateCommitRequest` | `crates/selene_kernel_contracts/src/ph1onb.rs` | request struct | FOUND | Stores access instance handoff fields. |
| `OnbCompleteCommitRequest` | `crates/selene_kernel_contracts/src/ph1onb.rs` | request struct | FOUND | Requires voice and optional wake receipt refs. |
| `OnbRequirementBackfillStartDraftRequest` | `crates/selene_kernel_contracts/src/ph1onb.rs` | request struct | FOUND | Starts backfill campaign. |
| `OnbRequirementBackfillNotifyCommitRequest` | `crates/selene_kernel_contracts/src/ph1onb.rs` | request struct | FOUND | Marks recipient requested after BCAST/REM handoff. |
| `OnbRequirementBackfillCompleteCommitRequest` | `crates/selene_kernel_contracts/src/ph1onb.rs` | request struct | FOUND | Completes backfill campaign. |
| `AppInviteLinkOpenRequest` | `crates/selene_os/src/app_ingress.rs` | app ingress request | FOUND | Canonical app invite-open request. |
| `AppOnboardingContinueRequest` | `crates/selene_os/src/app_ingress.rs` | app ingress request | FOUND | Canonical onboarding continuation request. |
| `InviteLinkOpenAdapterRequest` | `crates/selene_adapter/src/lib.rs` | adapter request | FOUND | JSON/API equivalent for invite click. |
| `OnboardingContinueAdapterRequest` | `crates/selene_adapter/src/lib.rs` | adapter request | FOUND | JSON/API equivalent for `/v1/onboarding/continue`. |

### Response structs and outcome equivalents

| Name | File / Path | Kind | Status | Notes |
|---|---|---|---|---|
| `OnbSessionStartResult` | `crates/selene_kernel_contracts/src/ph1onb.rs` | result struct | FOUND | Returns session id, status, next step, pinned schema context. |
| `OnbTermsAcceptResult` | `crates/selene_kernel_contracts/src/ph1onb.rs` | result struct | FOUND | Returns terms status. |
| `OnbEmployeePhotoCaptureSendResult` | `crates/selene_kernel_contracts/src/ph1onb.rs` | result struct | FOUND | Returns photo proof ref and verification pending status. |
| `OnbEmployeeSenderVerifyResult` | `crates/selene_kernel_contracts/src/ph1onb.rs` | result struct | FOUND | Returns verification status. |
| `OnbPrimaryDeviceConfirmResult` | `crates/selene_kernel_contracts/src/ph1onb.rs` | result struct | FOUND | Returns primary device confirmation result. |
| `OnbAccessInstanceCreateResult` | `crates/selene_kernel_contracts/src/ph1onb.rs` | result struct | FOUND | Returns access engine instance id. |
| `OnbCompleteResult` | `crates/selene_kernel_contracts/src/ph1onb.rs` | result struct | FOUND | Returns complete status. |
| `OnbRequirementBackfill*Result` | `crates/selene_kernel_contracts/src/ph1onb.rs` | result structs | FOUND | Backfill campaign/target progress results. |
| `AppInviteLinkOpenOutcome` | `crates/selene_os/src/app_ingress.rs` | app outcome | FOUND | Returns onboarding session, next step, required fields/gates. |
| `AppOnboardingContinueOutcome` | `crates/selene_os/src/app_ingress.rs` | app outcome | FOUND | Returns next step, blocking prompt, receipts, access id, status. |
| `InviteLinkOpenAdapterResponse` | `crates/selene_adapter/src/lib.rs` | adapter response | FOUND | HTTP/API bounded outcome. |
| `OnboardingContinueAdapterResponse` | `crates/selene_adapter/src/lib.rs` | adapter response | FOUND | HTTP/API bounded continuation response. |

### Records, enums, states, IDs, and tables

| Name | Type | Status | Notes |
|---|---|---|---|
| `OnboardingSessionRecord` | storage record | FOUND | PH1.F current-state row with token, invitee type, tenant, schema pinning, verification, receipts, missing-field state. |
| `OnboardingSessionId` | ID type | FOUND | Deterministic id derived from token hash in storage. |
| `BackfillCampaignId` | ID type | FOUND | Backfill campaign id. |
| `OnboardingStatus` | enum | FOUND | `DraftCreated`, `TermsAccepted`, `TermsDeclined`, `VerificationPending`, `VerificationConfirmed`, `VerificationRejected`, `PrimaryDeviceConfirmed`, `AccessInstanceCreated`, `Complete`. |
| `OnboardingNextStep` | enum | FOUND | `Install`, `Terms`, `LoadPrefilled`, `AskMissing`. |
| `AppOnboardingContinueNextStep` | enum | FOUND | `AskMissing`, `PlatformSetup`, `Terms`, `PrimaryDeviceConfirm`, `VoiceEnroll`, `WakeEnroll`, `SenderVerification`, `EmoPersonaLock`, `AccessProvision`, `Complete`, `Ready`, `Blocked`. |
| `TermsStatus` | enum | FOUND | `Accepted`, `Declined`. |
| `VerificationStatus` | enum | FOUND | `Pending`, `Confirmed`, `Rejected`. |
| `ProofType` | enum | FOUND | `Biometric`, `Passcode`. |
| `SenderVerifyDecision` | enum | FOUND | `Confirm`, `Reject`. |
| `BackfillCampaignState` | enum | FOUND | `DraftCreated`, `Running`, `Completed`, `Canceled`. |
| `BackfillTargetStatus` | enum | FOUND | `Pending`, `Requested`, `Reminded`, `Completed`, `Exempted`, `Failed`. |
| `onboarding_drafts` | SQL table | FOUND | LINK/onboarding draft table, not final ONB session table. |
| `onboarding_link_tokens` | SQL table | FOUND | LINK token table; PH1.ONB consumes activated token context. |
| `onboarding_requirement_backfill_campaigns` | SQL table | FOUND | Backfill campaign table in migration 0014. |
| `onboarding_requirement_backfill_targets` | SQL table | FOUND | Backfill target table in migration 0014. |
| `onboarding_sessions` | physical SQL migration | NOT_FOUND | Described in DB wiring and implemented in PH1.F current-state store; no standalone migration found in inspected migration search. |
| `voice_enrollment_sessions` | SQL table | FOUND | Voice enrollment session table with onboarding session id. |
| `wake_enrollment_sessions` | SQL table | FOUND | Wake enrollment session table with optional onboarding session id. |

### Error types and reason codes

| Name | Status | Notes |
|---|---|---|
| `StorageError::ContractViolation` | FOUND | Main runtime fail-closed error for invalid state/fields/order. |
| `StorageError::ForeignKeyViolation` | FOUND | Used for missing rows such as session/link ids. |
| `ONB_OK_SESSION_START_DRAFT` through `ONB_OK_REQUIREMENT_BACKFILL_COMPLETE_COMMIT` | FOUND | PH1.ONB reason code namespace in OS runtime. |
| `ONB_REFUSE_INVALID`, `ONB_REFUSE_NOT_FOUND` | FOUND | PH1.ONB refusal codes in OS runtime namespace. |
| String reason literals such as `ONB_ASK_MISSING_REQUIRED_BEFORE_TERMS`, `ONB_VOICE_ENROLL_REQUIRED_BEFORE_COMPLETE` | FOUND | App ingress/storage reason strings enforce order. |
| Unified published ONB reason-code registry | PARTIAL | Reason-code constants are present, but OS comments say global registry is not formalized. |

## 6. Onboarding Types And Product Functions

| Type / Function | Evidence Found | Current Behavior | Owner | Security Implications | Missing Design |
|---|---|---|---|---|---|
| invited onboarding | Strong evidence in app ingress, ONB runtime, LINK handoff, blueprints | Link click activates token, ONB starts session, continuation steps progress to ready/complete | PH1.LINK + PH1.ONB | Link must be activated and tenant/app context must match | Full visible UX and PH1.WRITE guidance boundary |
| self onboarding | No concrete PH1.ONB flow found | Not currently proven | UNKNOWN | Could bypass invite/access if invented incorrectly | NOT_FOUND |
| employee onboarding | Strong evidence via `InviteeType::Employee`, position schema pinning, required gates | Employee sessions use position/company prerequisites and schema-required gates | PH1.ONB + PH1.POSITION + Access | Access provisioning and role templates require governance proof | Access/Governance owner mapping needs reconciliation |
| customer onboarding | SQL invitee type exists; no complete PH1.ONB customer flow found | Draft/token can name customer invitee type, but dedicated steps not proved | PH1.LINK now; future customer owner | Risk of assuming customer access/workflow from token | PARTIAL |
| supplier onboarding | No runtime PH1.ONB evidence found | Not currently proven | UNKNOWN | Supplier onboarding may involve business permissions | NOT_FOUND |
| contractor onboarding | No distinct runtime flow found | Not currently proven | UNKNOWN | Contractor roles need access policy | NOT_FOUND |
| friend/personal connection onboarding | `FRIEND`, `FAMILY_MEMBER`, `ASSOCIATE` in SQL invitee type check | Link draft type exists; ONB runtime-specific journey not proved | PH1.LINK partial; future personal connection owner | Must not grant company access | PARTIAL |
| tenant onboarding | Tenant scope exists and is enforced | ONB sessions can carry tenant id and employee path checks active company/position | PH1.ONB + Access/Governance | Tenant mismatch fails closed | Full tenant bootstrap owner map is partial |
| workspace onboarding | No concrete runtime owner found | Not currently proven | UNKNOWN | Workspace binding/access risk | NOT_FOUND |
| role/access onboarding | ONB access instance creation path exists | Creates PH2 access instance after gates | PH1.ONB current; Access future/canonical policy owner | Role grant must not be raw ONB authority | PARTIAL / OWNER_GAP |
| device enrollment | Primary device confirmation and platform setup receipts exist | Device proof gates access/complete; platform receipts gate terms/next steps | PH1.ONB + device/presence owner future | Device proof is protected evidence | PARTIAL |
| voice enrollment | Runtime and DB migrations found | Voice lock and receipt required before complete | PH1.VOICE.ID + PH1.ONB orchestration | Voice ID evidence does not grant authority | FOUND |
| document/photo ID enrollment | Photo blob/proof refs and sender verification found | Schema-required employee photo/evidence refs can be stored | PH1.ONB for refs; artifact/document owner future | Raw document/proof privacy | PARTIAL |
| requirement backfill | Runtime, migrations, docs, tests found | Backfill campaigns/targets progress through start/notify/complete | PH1.ONB + PH1.BCAST/REM | Current staff updates require permission/delivery gating | Found for position requirements; full UX partial |
| onboarding reminder/follow-up | Blueprint references BCAST/REM handoff | ONB records notify after delivery/timing handoff | PH1.BCAST/PH1.REM | PH1.ONB must not send | PARTIAL |
| onboarding recovery/status/help | Runtime returns next step/status and deterministic reuse | Status primitives exist | PH1.ONB + PH1.WRITE future | Must be access-scoped | PARTIAL |

## 7. Interaction With PH1.LINK

PH1.LINK provides the activated link context that PH1.ONB consumes.

Current handoff evidence:

- `AppServerIngressRuntime::run_invite_link_open_and_start_onboarding` first reads the link, validates tenant scope, checks that both `LINK_INVITE_OPEN_ACTIVATE_COMMIT` and `ONB_SESSION_START_DRAFT` are active for the tenant, executes PH1.LINK activation, requires `LinkStatus::Activated`, and then executes PH1.ONB session start.
- `Ph1OnbOrchRuntime::start_session_from_link_activation` validates `LinkActivationResult`, requires `activation_status == LinkStatus::Activated`, requires app-open fields, and calls `ph1onb_session_start_draft`.
- `Ph1fStore::ph1onb_session_start_draft` verifies that the link exists, status is `Activated`, app-open fields match the activation record, and tenant scope is consistent.

PH1.LINK provides:

- `token_id`
- activation status
- optional prefilled context ref
- missing required fields
- app platform
- app instance id
- deep link nonce
- link opened timestamp
- link tenant/prefilled context

PH1.ONB consumes:

- activated token id
- prefilled context ref
- tenant/app-open context
- missing required field state through the link draft payload
- app/device context needed for phone-first onboarding.

PH1.ONB does not own:

- token signature verification
- token expiry
- revocation/blocking
- device binding during open/activate
- token creation
- token delivery.

Invalid, expired, revoked, blocked, or non-activated links should be stopped in PH1.LINK before ONB start. Repo truth also adds defense-in-depth: ONB session start refuses if the linked record is not `Activated` or app-open context does not match activation context.

Where PH1.LINK stops:

- after activation and app-open context handoff.

Where PH1.ONB begins:

- at deterministic onboarding session start from an already activated token.

## 8. Interaction With Access / Governance / Role / Permission

Current PH1.ONB access behavior is mixed.

Repo evidence shows:

- `ONB_ACCESS_INSTANCE_CREATE_COMMIT` exists as a PH1.ONB simulation id and request variant.
- `ph1onb_access_instance_create_commit` checks terms accepted, primary device confirmed, persona lock present, schema-required sender verification confirmed, employee position/company prerequisites, and then calls `ph2access_upsert_instance_commit`.
- It may insert a missing identity row before access instance creation.
- It maps invitee type to a default role id in app ingress before building `OnbAccessInstanceCreateCommitRequest`.
- DB wiring says Access gate decision output should guard governed ONB commit paths, and Access/Governance owns deny/escalate behavior.

Current conclusion:

- PH1.ONB currently coordinates access-instance creation as part of onboarding completion.
- Access instance storage truth is PH2 access storage.
- Full Access/Governance policy enforcement is `PARTIAL` in the extracted current path and must be reconciled.

Critical future rule:

Onboarding may collect and prepare access context, but Access/Governance must grant access. PH1.ONB must not grant authority by itself unless repo truth proves a bounded canonical access-owner path. Current repo truth proves a PH2 access storage write, not a complete future Access/Governance policy architecture.

Access-related fail-closed current checks include:

- tenant scope must match request/session/link scope.
- terms must be accepted.
- primary device proof must be confirmed.
- persona lock must exist.
- sender verification must be confirmed when required by pinned schema.
- employee company/position prerequisites must be active and consistent.
- access instance creation is idempotent by `(user_id, role_id, idempotency_key)`.

## 9. Interaction With Voice Identity / Human Presence

Voice enrollment is a real current onboarding dependency.

Repo evidence:

- `run_voice_enrollment_live_sequence` in `crates/selene_os/src/ph1onb.rs` routes enrollment through PH1.VOICE.ID simulation requests.
- App ingress `VoiceEnrollLock` action runs start/sample/complete sequence with multiple sample refs.
- `ph1onb_complete_commit` requires the latest locked voice enrollment for the onboarding session.
- It requires bound consent scope and a matching `voice_artifact_sync_receipt_ref`.
- Migration `0008_ph1vid_voice_enrollment_tables.sql` creates voice enrollment tables with onboarding-session fields.
- `docs/DB_WIRING/PH1_VOICE_ID.md` states voice enrollment is onboarding-session/device scoped and generates a receipt consumed by ONB completion.

Wake enrollment is also present:

- PH1.W migration has optional `onboarding_session_id`.
- `ph1onb_complete_commit` requires completed wake enrollment and matching receipt on wake-required platforms.
- iOS wake start is refused by default under explicit-trigger-only policy unless override is enabled; tests cover this.

Voice ID / Human Presence boundary:

- Voice ID provides enrollment, profile, speaker evidence, consent-scoped receipts, and device binding evidence.
- PH1.ONB gates completion on those receipts.
- Voice ID and wake enrollment do not grant authority by themselves.

Device identity:

- ONB records primary device proof and platform setup receipts.
- Device proof contributes to onboarding readiness but is not itself authority.

Document/photo proof:

- Employee photo capture proof refs exist.
- Full document/photo ID/KYC/liveness pipeline is `PARTIAL` or `NOT_FOUND` in PH1.ONB evidence.

## 10. Interaction With Broadcast / Delivery / Reminder

PH1.ONB does not send messages directly in the canonical extracted boundary.

Repo evidence:

- `docs/BLUEPRINTS/ONB_REQUIREMENT_BACKFILL.md` routes recipient notification through PH1.BCAST and reminder scheduling through PH1.REM before PH1.ONB records `ONB_REQUIREMENT_BACKFILL_NOTIFY_COMMIT`.
- `docs/DB_WIRING/PH1_ONB.md` explicitly says PH1.ONB does not deliver and does not schedule reminders directly.
- Backfill target status includes `Requested` and `Reminded`, but the outbound communication mechanics belong to BCAST/DELIVERY/REM.
- The invited onboarding blueprint mentions deferral/reminder behavior for "not now" and stopping/notify-sender behavior for "never"; concrete runtime delivery for those paths is not fully proven in ONB code.

Current conclusion:

- Backfill notification progress is present.
- Direct onboarding reminder/follow-up delivery is `PARTIAL`.
- PH1.ONB must hand outbound communication to PH1.BCAST/PH1.DELIVERY/PH1.REM.

## 11. PH1.WRITE / OpenAI / User Guidance Interaction

Current repo evidence shows deterministic and client-side wording more than a full PH1.WRITE boundary for onboarding.

Evidence:

- `AppOnboardingContinueOutcome` carries `blocking_field`, `blocking_question`, `next_step`, status, receipt refs, and access id.
- `onboarding_missing_field_question` in app ingress creates missing-field prompt text.
- Desktop and iPhone Swift surfaces contain explanatory UI text for onboarding states.
- Adapter returns structured status/outcome strings.
- No direct PH1.WRITE-owned onboarding guidance pipeline was found in the inspected PH1.ONB runtime path.
- No OpenAI/GPT-5.5 onboarding guidance proposal path was found in the current PH1.ONB runtime path.

Correct future rule:

OpenAI/GPT-5.5 may assist with onboarding explanation, clarification, and troubleshooting through PH1.D. PH1.WRITE owns final user-facing onboarding wording. PH1.ONB should not become the writing brain.

Current risks:

- `ONB_WRITING_OWNER_RISK`: current onboarding prompt/status wording is partly generated in app ingress and clients.
- `CLIENT_ONBOARDING_TEXT_RISK`: Desktop/iPhone contain explanatory user-facing onboarding text.
- `HARDCODED_ONBOARDING_GUIDANCE_RISK`: missing-field and route guidance text is deterministic/hardcoded rather than PH1.WRITE-owned.

These are not runtime changes in this task. They are future reconciliation targets.

## 12. Desktop / iPhone / Adapter Boundaries

### Desktop

Repo evidence:

- Desktop parses bounded app-open/invite-open context and dispatches `/v1/invite/click`.
- Desktop can dispatch `/v1/onboarding/continue` for bounded actions such as missing field submit, platform setup receipt, terms acceptance, photo capture send, sender verification, primary device confirmation, voice/wake enrollment actions, persona lock, access provision, pairing completion, and complete.
- Desktop text repeatedly states that the shell is non-authoritative and that canonical runtime routing owns onboarding state.

Current status:

- Desktop is more than read-only because it can submit bounded continuation actions.
- Desktop is not the current canonical ONB owner; it calls Adapter/runtime.
- `DESKTOP_ONB_AUTHORITY_RISK`: `PARTIAL`. The UI action surface is broad; future proof must show it cannot decide onboarding state, access, or authority locally.

### iPhone

Repo evidence:

- iPhone has `onboardingEntryActive`, invite/open parsing, outcome rows, onboarding continue preview rows, and read-only posture text.
- iPhone text says no invite activation, no onboarding mutation, no local authority, and no transcript mutation occur locally in the visible surface.

Current status:

- iPhone appears render/read-only for onboarding entry and status in inspected evidence.
- `IPHONE_ONB_AUTHORITY_RISK`: not found as a concrete current mutation path, but still needs render-only proof in future acceptance.

### Adapter

Repo evidence:

- Adapter exposes `/v1/invite/click` and `/v1/onboarding/continue`.
- HTTP adapter enforces bearer/security headers, request id, timestamp, nonce, expected subject, expected device, and idempotency key before calling runtime.
- Adapter parses string actions into `AppOnboardingContinueAction`.
- Adapter returns bounded response fields and error/security responses.

Current status:

- Adapter transports and validates request shape/security.
- Adapter must not become ONB state owner.
- `ADAPTER_ONB_AUTHORITY_RISK`: `PARTIAL` because it parses action strings and guards ingress, but canonical state mutation is runtime/storage-owned.

## 13. Security / Privacy / Consent Model

Current security evidence:

- Tenant scope is checked against request, link prefilled context, and onboarding session.
- App-open context is mandatory for invite-click onboarding start.
- App instance id, deep link nonce, and link opened time must match PH1.LINK activation context.
- Device fingerprint is hashed in ONB session state.
- Session start is one deterministic session per activated token.
- Idempotency indexes exist for terms, photo, sender verification, primary device, access create, complete, and backfill.
- Terms acceptance is required before later steps.
- Primary device proof is required before voice/persona/access/complete.
- Runtime governance checks primary device proof as protected-action proof.
- Voice enrollment consent and sync receipt are required before completion.
- Wake receipt is required on wake-required platforms.
- Employee position/company prerequisites are checked for employee access creation and completion.
- Link is consumed on completion.
- PH1.J audit transition events are emitted by PH1.ONB runtime.

Current privacy/design gaps:

- Full consent model for all onboarding data is `PARTIAL`.
- Photo/document proof raw artifact ownership is `PARTIAL`.
- KYC/photo ID/document verification is `NOT_FOUND` as a full ONB runtime flow.
- Wrong recipient/wrong role correction is not a complete ONB flow.
- Onboarding cancellation/expiry/recovery state model is `PARTIAL`.
- Physical SQL persistence for `onboarding_sessions` is `PARTIAL` because current DB wiring says PH1.F current-state runtime slice and migration search did not find an `onboarding_sessions` create table.

Critical rule:

Opening a link, hearing a voice, completing onboarding steps, or submitting a client action must not automatically grant authority. Access and authority remain deterministic governance concerns.

## 14. Onboarding State Machine

### Actual current ONB status enum

`OnboardingStatus` currently includes:

- `DraftCreated`
- `TermsAccepted`
- `TermsDeclined`
- `VerificationPending`
- `VerificationConfirmed`
- `VerificationRejected`
- `PrimaryDeviceConfirmed`
- `AccessInstanceCreated`
- `Complete`

### Actual app continuation next-step enum

`AppOnboardingContinueNextStep` currently includes:

- `AskMissing`
- `PlatformSetup`
- `Terms`
- `PrimaryDeviceConfirm`
- `VoiceEnroll`
- `WakeEnroll`
- `SenderVerification`
- `EmoPersonaLock`
- `AccessProvision`
- `Complete`
- `Ready`
- `Blocked`

### Backfill states

`BackfillCampaignState`:

- `DraftCreated`
- `Running`
- `Completed`
- `Canceled`

`BackfillTargetStatus`:

- `Pending`
- `Requested`
- `Reminded`
- `Completed`
- `Exempted`
- `Failed`

### RECONSTRUCTED_FROM_REPO_EVIDENCE state machine

This is a repo-truth reconstruction, not a claim that every label exists as a runtime enum:

1. `INVITED`
   - PH1.LINK has created a token/draft.
2. `LINK_ACTIVATED`
   - PH1.LINK open/activate returned `Activated`.
3. `SESSION_STARTED`
   - PH1.ONB wrote `DraftCreated`.
4. `WAITING_FOR_FIELDS`
   - app next step is `AskMissing`.
5. `WAITING_FOR_PLATFORM_SETUP`
   - app next step is `PlatformSetup`.
6. `WAITING_FOR_TERMS`
   - app next step is `Terms`.
7. `TERMS_ACCEPTED` or `TERMS_DECLINED`
   - status is `TermsAccepted` or `TermsDeclined`; declined leads to blocked.
8. `PENDING_VERIFICATION`
   - schema-required photo/sender verification path.
9. `PRIMARY_DEVICE_CONFIRMED`
   - device proof accepted.
10. `PENDING_VOICE_ENROLLMENT`
   - voice lock still missing.
11. `PENDING_WAKE_ENROLLMENT`
   - wake-required platform still needs wake receipt.
12. `PENDING_PERSONA_LOCK`
   - persona/tone setup lock still missing.
13. `PENDING_ACCESS_PROVISION`
   - access instance not yet created.
14. `READY_TO_COMPLETE`
   - all prerequisites present.
15. `COMPLETED`
   - status is `Complete`; link consumed.
16. `BLOCKED`
   - terms declined, verification rejected, failed proof, missing simulation, invalid state, or policy refusal.

Missing explicit enum states:

- `NOT_STARTED`
- `INVITED`
- `LINK_ACTIVATED`
- `IN_PROGRESS`
- `PENDING_ACCESS_APPROVAL`
- `PENDING_DEVICE_ENROLLMENT`
- `BACKFILL_REQUIRED`
- `EXPIRED`
- `CANCELLED`
- `FAILED`

These missing labels should not be claimed as current runtime states without future proof.

## 15. Error Handling And Reason Codes

Existing error/refusal patterns:

| Error / Reason | Evidence | Status | Notes |
|---|---|---|---|
| invalid link activation | PH1.LINK response handling, activation status check | FOUND | ONB bridge refuses non-activated activation result. |
| non-activated token | `ph1onb_session_start_draft.link_status` | FOUND | Link must be `Activated`. |
| missing required field | `ph1onb_ask_missing_field_turn`, `AskMissing` | FOUND | Missing field prompts and repeated failures are tracked. |
| invalid field | LINK draft update contract violation in ask-missing | FOUND | Failed update increments attempts and can escalate. |
| tenant mismatch | app ingress and storage checks | FOUND | Request/session/link tenant mismatches fail closed. |
| workspace mismatch | no workspace ONB runtime found | NOT_FOUND | Future workspace onboarding needs owner proof. |
| access denied | DB wiring references Access gate; runtime access create has prerequisites | PARTIAL | Full Access/Governance deny/escalate path needs reconciliation. |
| role conflict | default role mapping and access instance path | PARTIAL | Role conflict reason code not found as first-class ONB code. |
| duplicate onboarding session | deterministic reuse by token | FOUND | Existing session reused if app-open context matches. |
| expired onboarding | no explicit ONB expiry state found | DESIGN_GAP | Link expiry is PH1.LINK-owned before activation. |
| cancelled onboarding | terms decline and blocked exist; general cancel missing | PARTIAL | Cancellation state/API not found. |
| missing simulation | app ingress/executor uses `SIM_DISPATCH_GUARD_SIMULATION_NOT_REGISTERED` and `SIM_DISPATCH_GUARD_SIMULATION_NOT_ACTIVE` | FOUND | Active simulation checks guard invite click and continuation actions. |
| missing identity proof | photo/sender/device/voice checks exist | PARTIAL | Full identity proof/KYC flow not found. |
| missing voice enrollment | `ONB_VOICE_ENROLL_REQUIRED_BEFORE_*` | FOUND | Access/complete/persona lock require voice lock in current continuation path. |
| device mismatch | activation/app-open and voice enroll device checks | FOUND | Device/app context and primary device mismatch fail closed. |
| onboarding already completed | idempotency complete replay exists | PARTIAL | No broad already-completed UX path found. |
| unsupported onboarding type | no full type matrix in runtime | PARTIAL | Customer/supplier/etc. not fully implemented. |
| client route mismatch | adapter ingress security and route parsing | PARTIAL | Security guards exist; full client compatibility retirement needs future proof. |

Bounded PH1.ONB reason-code constants currently found:

- `ONB_OK_SESSION_START_DRAFT`
- `ONB_OK_TERMS_ACCEPT_COMMIT`
- `ONB_OK_EMPLOYEE_PHOTO_COMMIT`
- `ONB_OK_EMPLOYEE_SENDER_VERIFY_COMMIT`
- `ONB_OK_PRIMARY_DEVICE_CONFIRM_COMMIT`
- `ONB_OK_ACCESS_INSTANCE_CREATE_COMMIT`
- `ONB_OK_COMPLETE_COMMIT`
- `ONB_OK_REQUIREMENT_BACKFILL_START_DRAFT`
- `ONB_OK_REQUIREMENT_BACKFILL_NOTIFY_COMMIT`
- `ONB_OK_REQUIREMENT_BACKFILL_COMPLETE_COMMIT`
- `ONB_REFUSE_INVALID`
- `ONB_REFUSE_NOT_FOUND`

Runtime string refusal examples include:

- `ONB_ASK_MISSING_REPEAT_ESCALATION`
- `ONB_ASK_MISSING_REQUIRED_BEFORE_TERMS`
- `ONB_PLATFORM_SETUP_REQUIRED_BEFORE_TERMS`
- `ONB_PRIMARY_DEVICE_CONFIRM_REQUIRED_BEFORE_VOICE_ENROLL`
- `ONB_SENDER_VERIFICATION_REQUIRED_BEFORE_VOICE_ENROLL`
- `ONB_VOICE_ENROLL_REQUIRED_BEFORE_ACCESS_PROVISION`
- `ONB_EMO_PERSONA_LOCK_REQUIRED_BEFORE_ACCESS_PROVISION`
- `ONB_ACCESS_PROVISION_REQUIRED_BEFORE_COMPLETE`
- `ONB_WAKE_ENROLL_REQUIRED_BEFORE_COMPLETE`

## 16. Audit / Provenance / Evidence

PH1.ONB audit is present but not complete across all future product needs.

Current audit evidence:

- `Ph1OnbOrchRuntime::audit_transition` emits PH1.J audit events using `AuditEngine::Other("ph1_onb")`, `AuditEventType::StateTransition`, and bounded payload containing `state_from`, `state_to`, and optional evidence ref.
- Session start audit uses `ONB_OK_SESSION_START_DRAFT`.
- Terms audit uses `ONB_OK_TERMS_ACCEPT_COMMIT`.
- Photo audit uses `ONB_OK_EMPLOYEE_PHOTO_COMMIT`.
- Sender verify audit uses `ONB_OK_EMPLOYEE_SENDER_VERIFY_COMMIT`.
- Primary device audit uses `ONB_OK_PRIMARY_DEVICE_CONFIRM_COMMIT`.
- Access instance audit uses `ONB_OK_ACCESS_INSTANCE_CREATE_COMMIT`.
- Complete audit uses `ONB_OK_COMPLETE_COMMIT`.
- Backfill start/notify/complete audit uses the corresponding backfill reason codes.
- Storage records retain ids, receipts, statuses, tenant, device/app-open context, missing-field state, access instance id, and proof refs.

Required audit questions:

| Question | Current Answer | Status |
|---|---|---|
| Is onboarding session start audited? | Yes, PH1.J transition in ONB runtime. | FOUND |
| Is link activation handoff audited? | LINK owns activation audit; ONB audits session start after handoff. | PARTIAL |
| Are field submissions audited? | Missing-field state is stored; explicit PH1.J field-submission audit not found in ONB runtime. | AUDIT_GAP |
| Are missing-field prompts audited? | Stored in session state; no dedicated PH1.J prompt event found. | PARTIAL |
| Are verification steps audited? | Photo/sender/device transitions are audited. | FOUND |
| Are access/role handoffs audited? | ONB access instance transition audited; Access owner audit needs reconciliation. | PARTIAL |
| Are voice/device enrollment handoffs audited? | Voice/wake owners record their own state; ONB completion checks receipts. | PARTIAL |
| Are reminders/follow-ups audited? | Backfill notify/progress audited; delivery/reminder audit belongs to BCAST/REM. | PARTIAL |
| Are failed/cancelled/expired attempts audited? | Contract errors/refusals exist; comprehensive failed/cancelled ONB audit not found. | AUDIT_GAP |
| Are client/device/session refs recorded? | Device/app-open context and session ids are recorded. | FOUND |
| Are tenant/workspace/user refs recorded? | Tenant/user refs exist; workspace refs not found. | PARTIAL |

## 17. Current Tests / Smokes / Acceptance

| Test / Smoke | Path | What It Proves | What It Does Not Prove | Status |
|---|---|---|---|---|
| `at_onb_db_01_tenant_isolation_enforced` | `crates/selene_storage/tests/ph1_onb/db_wiring.rs` | Tenant mismatch fails closed for session start. | Full Access/Governance policy. | FOUND |
| `at_onb_db_01b_phone_first_start_requires_exact_link_open_activate_handoff_context` | `crates/selene_storage/tests/ph1_onb/db_wiring.rs` | Phone-first app-open context must match activation. | UI behavior. | FOUND |
| `at_onb_db_02_append_only_enforced` | `crates/selene_storage/tests/ph1_onb/db_wiring.rs` | Append-only/current-state discipline for row 21. | Full SQL physical table. | FOUND |
| `at_onb_db_03_idempotency_dedupe_works` | `crates/selene_storage/tests/ph1_onb/db_wiring.rs` | Retried commits return deterministic prior result. | External delivery retries. | FOUND |
| `at_onb_db_04_current_table_no_ledger_rebuild_required` | `crates/selene_storage/tests/ph1_onb/db_wiring.rs` | Current-state ONB storage is sufficient for row 21. | SQL `onboarding_sessions` migration. | FOUND |
| `at_onb_db_05_session_start_pins_schema_context_and_required_gates` | `crates/selene_storage/tests/ph1_onb/db_wiring.rs` | Pinned schema context and required verification gates. | Full schema management UX. | FOUND |
| `at_onb_db_06_required_sender_verification_blocks_access_and_complete_until_confirmed` | `crates/selene_storage/tests/ph1_onb/db_wiring.rs` | Access/complete blocked until required sender verification confirmed. | Future approval workflows. | FOUND |
| `at_onb_db_07_photo_sender_commits_refuse_when_schema_gate_not_required` | `crates/selene_storage/tests/ph1_onb/db_wiring.rs` | Photo/sender commits require pinned schema gate. | Raw photo/document handling. | FOUND |
| `at_onb_db_12_required_verification_commit_idempotency_replays_deterministically` | `crates/selene_storage/tests/ph1_onb/db_wiring.rs` | Verification commit idempotency. | External verification providers. | FOUND |
| `at_onb_db_13_ios_complete_allows_missing_wake_receipt_when_voice_locked` | `crates/selene_storage/tests/ph1_onb/db_wiring.rs` | iOS can complete without wake receipt when voice locked. | Feature-flag override policy UX. | FOUND |
| `at_onb_db_14_android_complete_requires_wake_receipt_when_wake_is_complete` | `crates/selene_storage/tests/ph1_onb/db_wiring.rs` | Android wake receipt requirement. | Native Android client proof. | FOUND |
| `at_onb_db_14b_desktop_complete_requires_wake_receipt_when_wake_is_complete` | `crates/selene_storage/tests/ph1_onb/db_wiring.rs` | Desktop wake receipt requirement. | Native Desktop wake proof. | FOUND |
| `at_onb_db_15_wake_start_refused_for_ios_onboarding_session_default_policy` | `crates/selene_storage/tests/ph1_onb/db_wiring.rs` | iOS wake start refused by default. | Policy UI messaging. | FOUND |
| `at_onb_db_16_wake_start_allows_ios_override_flag` | `crates/selene_storage/tests/ph1_onb/db_wiring.rs` | Policy override can allow iOS wake start. | Governance approval flow. | FOUND |
| `at_onb_db_08` through `at_onb_db_11` backfill tests | `crates/selene_storage/tests/ph1_onb/db_wiring.rs` | Backfill scope, deterministic snapshot, idempotent notify/complete, tenant/missing target failures. | Actual BCAST/REM external delivery. | FOUND |
| `runc_onb_db_ask_missing_state_round_trip_updates_session_record` | `crates/selene_storage/tests/ph1_onb/db_wiring.rs` | Ask-missing state persists and updates session. | PH1.WRITE guidance. | FOUND |
| `rund_onb_db_platform_setup_receipt_round_trip_updates_session_record` | `crates/selene_storage/tests/ph1_onb/db_wiring.rs` | Platform setup receipt maps persist. | Full client receipt authenticity. | FOUND |
| `run1_invite_link_click_starts_onboarding_with_active_simulations` | `crates/selene_os/src/app_ingress.rs` | Invite click starts ONB when LINK/ONB simulations active. | Full external HTTP security. | FOUND |
| `run1_invite_link_click_fails_closed_when_onboarding_simulation_not_active` | `crates/selene_os/src/app_ingress.rs` | Missing active ONB simulation blocks onboarding start. | User-facing explanation. | FOUND |
| onboarding continuation tests in app ingress | `crates/selene_os/src/app_ingress.rs` | Missing fields, receipts, terms, device, voice, wake, persona, access, complete sequencing. | End-user visual UX. | FOUND |
| onboarding continue adapter tests | `crates/selene_adapter/src/lib.rs` | Adapter action parsing and progression through terms/device/voice/sender verification/wake. | Runtime authority owner proof beyond Adapter boundary. | FOUND |
| HTTP onboarding security tests | `crates/selene_adapter/src/bin/http_adapter.rs` | `/v1/onboarding/continue` requires bearer/security headers. | Full production auth deployment. | FOUND |
| iPhone invite onboarding E2E | `crates/selene_adapter/src/bin/http_adapter.rs` | Invite click and iPhone onboarding path can progress through adapter/runtime. | Native iPhone live UI proof. | FOUND |
| Desktop visible proof | Swift source evidence only in this extraction | Desktop routes and non-authority text exist. | Automated Desktop UI acceptance not extracted here. | PARTIAL / TEST_GAP |

## 18. Old Paths / Compatibility / Wrong-Owner Risks

| Path / Symbol | Current Status | Correct Canonical Owner | Retirement Condition | Active-Caller Check Needed |
|---|---|---|---|---|
| `PH1.ONB.CORE.001`, `PH1.ONB.ORCH`, `PH1.ONB.ORCH.001`, `PH1.ONB.BIZ.001` | Legacy/spec aliases per DB wiring | PH1.ONB single wired engine surface | After docs/code references map cleanly to canonical PH1.ONB | Yes |
| `ONB_DRAFT_UPDATE_COMMIT` | Present in simulation catalog/blueprint; current app path uses `LINK_INVITE_DRAFT_UPDATE_COMMIT` for ask-missing updates | PH1.LINK draft update or future canonical ONB missing-field owner | After reconciliation resolves exact owner | Yes |
| `ONB_EMPLOYEE_PHOTO_CAPTURE_SEND_COMMIT` | Legacy id retained; storage writes proof refs, not delivery | PH1.ONB evidence refs; BCAST/DELIVERY for sending | After proof that no external send occurs through ONB | Yes |
| `/v1/invite/click` | Active adapter route | Adapter transport; PH1.LINK + PH1.ONB runtime | Retain; not an old path | Yes for route security |
| `/v1/onboarding/continue` | Active adapter route | Adapter transport; PH1.ONB runtime | Retain; not an old path | Yes for action scope |
| iPhone `inviteLike`, `openLike`, `appOpenLike` parsing | Compatibility/route classification in client | iPhone render/open only | After canonical deep-link parser proof | Yes |
| Desktop bounded onboarding action builders | Active client-to-adapter route builders | Desktop client only; Adapter/runtime decide | Keep only if all actions are bounded and canonical | Yes |
| ONB direct access instance creation | Active current path | Access/Governance policy owner + PH2 access storage | After Access/Governance path is proven and ONB role narrowed | Yes |
| app ingress hardcoded missing-field prompts | Active deterministic guidance | PH1.WRITE future | After PH1.WRITE onboarding guidance boundary exists | Yes |
| business setup `ONB_BIZ_*` blueprint | Active blueprint, runtime ownership points to PH1.POSITION for company rows | PH1.POSITION + Access/Governance, with ONB only where session/backfill owner | After repo-truth build maps current callers | Yes |
| no standalone `crates/selene_engines/src/ph1onb.rs` | Current implementation shape | OS/storage ONB runtime | No retirement needed | No |
| no SQL `onboarding_sessions` migration found | Current storage gap | PH1.F storage / future SQL owner | After persistence strategy is settled | Yes |
| client onboarding text | Active Swift UI text | PH1.WRITE future for final guidance | After PH1.WRITE owns visible onboarding wording | Yes |

## 19. Full Functionality Master List

| Functionality | Description | Evidence | Owner | Status | Future Action |
|---|---|---|---|---|---|
| start invited onboarding | Create/reuse onboarding session after activated invite link | app ingress, PH1.ONB runtime, storage | PH1.LINK + PH1.ONB | FOUND | Preserve owner split. |
| validate activated link context | ONB refuses non-activated link and mismatched app-open context | storage/app ingress | PH1.LINK primary, PH1.ONB defensive | FOUND | Keep defense-in-depth. |
| compute pinned schema context | Pin schema id/version/overlay/selector and required gates | storage tests, DB wiring | PH1.ONB execution; PH1.POSITION schema owner | FOUND | Reconcile non-position schema registry. |
| compute missing required fields | Store missing fields from link draft/pinned payload | storage | PH1.ONB + PH1.LINK draft | FOUND | Move guidance to PH1.WRITE later. |
| ask missing field | Prompt/update/escalate missing field state | `ph1onb_ask_missing_field_turn` | PH1.ONB | FOUND | Add better PH1.WRITE/PH1.N integration later. |
| platform setup receipt | Accept bounded platform setup receipts | app ingress/storage | PH1.ONB + client evidence | FOUND | Prove client authenticity/security. |
| accept/decline terms | Record terms status | ONB contract/runtime/storage | PH1.ONB | FOUND | Connect final wording to PH1.WRITE. |
| photo/evidence proof refs | Record schema-required photo proof refs | ONB contract/runtime/storage | PH1.ONB, artifact owner future | FOUND/PARTIAL | Define raw artifact owner. |
| sender verification | Confirm/reject schema-required sender verification | ONB contract/runtime/storage | PH1.ONB | FOUND | Connect notification/approval owner if needed. |
| primary device proof | Record primary device proof | ONB contract/runtime/storage | PH1.ONB + device/presence owner future | FOUND | Preserve governance proof check. |
| voice enrollment orchestration | Run PH1.VOICE.ID enrollment start/sample/complete | PH1.ONB runtime/app ingress | PH1.VOICE.ID, ONB orchestration | FOUND | Keep Voice ID evidence-only law. |
| wake enrollment orchestration | Run PH1.W enrollment where required | app ingress/storage tests | PH1.W, ONB orchestration | FOUND | Keep platform policy proof. |
| persona/tone lock | Run tone-only PH1.EMO/PH1.PERSONA path and store audit id | app ingress/storage | PH1.EMO/PH1.PERSONA + ONB lock | FOUND | Reconcile with Selene Emotional docs. |
| access provisioning | Create PH2 access instance after ONB gates | storage/app ingress | Current ONB; future Access/Governance policy | PARTIAL | Verify canonical access owner. |
| complete onboarding | Mark complete, store receipts, consume link | ONB runtime/storage | PH1.ONB | FOUND | Add user-facing completion through PH1.WRITE. |
| consume link on completion | Set link status consumed if activated/opened | storage | PH1.ONB completion affecting PH1.LINK record | FOUND | Reconcile cross-owner state mutation. |
| start backfill campaign | Create deterministic campaign/target snapshot | storage/docs/tests | PH1.ONB | FOUND | Wire user journey later. |
| notify backfill target | Mark target requested after delivery/reminder handoff | storage/docs/tests | PH1.ONB progress; BCAST/REM communication | FOUND/PARTIAL | Prove live delivery handoff. |
| complete backfill campaign | Mark campaign complete | storage/docs/tests | PH1.ONB | FOUND | Add status assistant. |
| adapter invite click | POST `/v1/invite/click` | HTTP adapter | Adapter transport | FOUND | Keep transport-only. |
| adapter onboarding continue | POST `/v1/onboarding/continue` | HTTP adapter | Adapter transport | FOUND | Keep action parsing bounded. |
| Desktop invite/open UI | Dispatch invite click and render outcome | Swift Desktop | Desktop client | FOUND | Prove no local authority. |
| iPhone onboarding entry UI | Render bounded onboarding entry/preview | Swift iPhone | iPhone client | FOUND | Prove render-only. |
| onboarding reminders | Deferral/backfill docs | BCAST/REM future | PARTIAL | Build through BCAST/REM, not ONB. |
| customer/supplier onboarding | Not concretely implemented | Search evidence | UNKNOWN | NOT_FOUND/PARTIAL | Future design needed. |
| workspace onboarding | Not concretely implemented | Search evidence | UNKNOWN | NOT_FOUND | Future owner needed. |
| cancellation/recovery | Terms decline/blocking and idempotent reuse only | runtime/docs | PH1.ONB partial | PARTIAL | Explicit state model needed. |

## 20. Comparison To Master Architecture

### Global Request Decision Lattice

Current PH1.ONB is mostly execution/continuation state, not the user-meaning router. Future onboarding requests should be classified by PH1.X before creation, access handoff, reminders, or protected execution. Current app ingress continuation actions are deterministic API actions, not natural-language request routing.

### PH1.D Proposal Gateway

No direct PH1.D/OpenAI onboarding proposal gateway was found in current ONB runtime. Future messy onboarding explanations, missing-field help, and troubleshooting should use PH1.D only as governed proposal source, never authority.

### PH1.N Meaning Unravelling

Current ONB missing-field behavior is deterministic. Future natural onboarding help should let PH1.N propose meaning candidates while PH1.X/ONB validate required fields and gates.

### PH1.WRITE Human Presentation

Current onboarding guidance/status wording is partly app-ingress/client/adapter text. Future architecture should route final onboarding explanations through PH1.WRITE, especially for blocked, failed, sensitive, or access-impacting onboarding states.

### PH1.LINK Link Journey

Current ONB correctly starts after PH1.LINK activation. Link Journey Intelligence should remain above LINK/ONB: it resolves the human request, confirms link generation/sending, and then PH1.LINK/PH1.ONB do their canonical mechanics.

### Broadcast / Delivery / Reminder

Current backfill docs require BCAST/REM handoff before ONB notify progress. ONB must not send messages, reminders, or notifications directly.

### Voice Identity + Human Presence

Current ONB has strong voice enrollment linkage and requires locked voice receipt before complete. Voice ID remains evidence. Voice enrollment must not grant authority.

### Identity + Access + Authority Spine

Current ONB access provisioning touches PH2 access storage. Future reconciliation must ensure Access/Governance owns access policy and Authority owns protected action permission. ONB should coordinate onboarding readiness, not become authority.

### Master Access Template / Role / Permission stack

Current default role mapping exists in runtime. Future role/access template resolution must move through Access/Governance and approved role templates, especially for employee/executive/high-access onboarding.

### Tenant / Workspace Governance

Tenant scope is strongly represented. Workspace scope is not found as a concrete ONB runtime feature and needs future owner mapping.

### Desktop/iPhone render-only boundary

iPhone appears read-only in inspected evidence. Desktop has bounded continuation actions but routes through Adapter/runtime and states non-authoritative posture. Both need future live proof that they cannot decide access/onboarding state locally.

### Adapter transport-only boundary

Adapter translates JSON and enforces ingress security. It must stay transport/shape/security, not onboarding state owner.

### Old Compatibility Path Retirement

Legacy aliases, blueprint-only IDs, client route heuristics, and cross-owner access provisioning should be mapped in a retirement ledger before any deletion.

## 21. Gaps / Missing Pieces

| Gap | Evidence | Risk | Recommended Future Action | Priority |
|---|---|---|---|---|
| missing onboarding type matrix | Employee path is strong; customer/supplier/friend/workspace flows partial/not found | Incorrectly assuming unsupported onboarding types | Build PH1.ONB type matrix activation pack | High |
| access template integration partial | ONB calls PH2 access storage directly with default role mapping | ONB could be mistaken for access policy owner | Reconcile Access/Governance and role-template ownership | Critical |
| workspace onboarding missing | No workspace-specific ONB runtime path found | Workspace access could be guessed from tenant/link | Add workspace owner discovery before design | High |
| voice enrollment proof exists but UX/policy incomplete | Runtime and tests exist; user-facing consent flow partial | Voice enrollment may appear to grant authority | Voice enrollment handoff proof and consent UI pack | High |
| device enrollment owner partial | Primary device proof/platform receipts exist | Device trust could be overclaimed | Device/Human Presence owner mapping | High |
| document/photo proof owner partial | Photo blob/proof refs exist; raw artifact owner absent | Sensitive proof leakage or wrong owner | Artifact/document proof admission design | High |
| consent model partial | Voice consent field exists; broad onboarding data consent incomplete | Privacy and legal risk | Onboarding consent matrix and PH1.WRITE wording | High |
| PH1.WRITE boundary missing | Hardcoded/app/client text found | Inconsistent or unsafe user guidance | PH1.WRITE onboarding guidance boundary slice | Medium |
| onboarding reminders/follow-up proof partial | Backfill docs reference BCAST/REM | ONB might wrongly send messages | BCAST/REM onboarding notification proof | Medium |
| audit gaps for prompts/failures | State transition audit exists; failed/cancelled/prompt audit partial | Thin forensic evidence | ONB audit evidence pack | Medium |
| SQL persistence gap | No physical `onboarding_sessions` create table found | Runtime state not matched to SQL schema | Decide SQL current-state table strategy | High |
| client render-only proof partial | iPhone read-only; Desktop submits bounded actions | Client wrong-owner risk | Desktop/iPhone live render/action boundary proof | High |
| Adapter transport-only proof partial | Adapter parses action strings and enforces security | Adapter may accrete business logic | Adapter ONB transport-only proof | Medium |
| old blueprint/runtime mismatch | `ONB_DRAFT_UPDATE_COMMIT` vs LINK draft update | Duplicate owner confusion | Compatibility ledger and canonical owner mapping | Medium |
| JD live acceptance missing | No live proof extracted | Product could be technically correct but UX unverified | JD live onboarding acceptance pack | High |

## 22. Recommended Future Build Slices

Based on repo truth, recommended future slices are:

1. PH1.ONB Repo-Truth Activation Pack
2. Onboarding Contract / State Machine Normalization
3. Invited Onboarding From PH1.LINK Activation Proof
4. Required Field / Missing Field Flow
5. PH1.WRITE Onboarding Guidance Boundary
6. Access Template / Role / Permission Handoff
7. Tenant / Workspace Scope Integration
8. Voice Enrollment Handoff Proof
9. Device Enrollment Handoff Proof
10. Onboarding Reminder / Follow-Up Handoff
11. Desktop/iPhone Render-Only Onboarding Proof
12. Adapter Transport-Only Onboarding Proof
13. Onboarding Audit Evidence Pack
14. Old Onboarding Compatibility Retirement Ledger
15. JD Live Onboarding Acceptance Pack

Use actual repo findings before implementation. Do not implement from this extraction document alone.

## 23. What Codex Must Not Do

Codex must not:

- invent onboarding behavior
- create duplicate onboarding engine
- grant access from onboarding without Access/Governance
- grant authority from onboarding
- bypass PH1.LINK activation for invited onboarding
- send messages directly from PH1.ONB
- bind Voice ID authority from onboarding
- let Desktop/iPhone decide onboarding state
- let Adapter decide onboarding state
- bypass PH1.WRITE for user-facing guidance where unsafe
- delete old paths before proof
- implement from this extraction document alone
- treat SQL draft/link tables as proof of every future onboarding product type
- treat current access instance creation as full future access policy
- treat voice enrollment as identity/access/authority
- treat provider/OpenAI output as onboarding truth

## 24. Final Extracted Architecture Sentence

PH1.ONB is Selene's governed onboarding-session boundary: it may collect required onboarding fields, start invited onboarding after PH1.LINK activation, coordinate setup steps, and hand off to Access, Voice Identity, Delivery, and other canonical owners, but it must not create links, send messages, grant authority, or bypass access governance.
