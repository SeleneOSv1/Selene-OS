# Selene Master Access Engine — Repo-Truth Functionality Extraction Master Design

Document status:

- REPO_TRUTH_EXTRACTION
- NOT_RUNTIME_IMPLEMENTATION
- PENDING_GRAND_ARCHITECTURE_RECONCILIATION

## 0. Authority and Scope

AGENTS.md controls execution.

This is docs-only repo-truth extraction.

No runtime code was changed.

This document does not authorize implementation.

The document reconstructs current Master Access / Access Governance / Role / Permission / Authority boundary design and functionality from repo evidence.

Future implementation/refactor/retirement requires explicit build instruction, approved file scope, tests, backend evidence, JD approval, and where visible JD live proof.

## 1. Executive Summary

The current Master Access system is not a single standalone runtime file named `ph1access.rs` under `crates/selene_os` or `crates/selene_engines`. Repo truth shows an access system split across:

- `crates/selene_kernel_contracts/src/ph1access.rs` for PH1.ACCESS master schema contracts.
- `crates/selene_storage/src/ph1f.rs` and `crates/selene_storage/src/repo.rs` for PH1.ACCESS/PH2.ACCESS storage truth, per-user access instances, AP schema ledgers, overlays, board policies, votes, overrides, and gate decisions.
- `crates/selene_storage/migrations/0009_access_instance_tables.sql`, `0015_access_master_schema_tables.sql`, and `0016_access_ap_authoring_review_tables.sql` for SQL table shape.
- `docs/DB_WIRING/PH1_ACCESS_001_PH2_ACCESS_002.md` and `docs/ECM/PH1_ACCESS_001_PH2_ACCESS_002.md` for the canonical DB wiring and capability map.
- `crates/selene_os/src/simulation_executor.rs` for OS-side access gate enforcement before certain simulation candidate dispatch paths.

Current repo owner naming is `PH1.ACCESS.001_PH2.ACCESS.002`. PH1.ACCESS appears to own deterministic gate/read/schema-chain decision behavior; PH2.ACCESS appears to own per-user access instance and override writes. This split is explicit in DB wiring and ECM, while storage currently implements both under PH1.F.

Access and Authority are distinct. Access determines whether a user/access instance/scope/mode/action is allowed, denied, or requires escalation. Authority is the broader protected-execution posture that also needs identity, confirmation, policy, simulation certification, and runtime governance. Access is necessary for governed execution but is not sufficient by itself.

Access and Simulation are distinct. Access gates a requested path; SimulationExecutor then requires a registered/active simulation before committing protected work. Access does not execute protected business actions.

Access and Onboarding are distinct. PH1.ONB can currently create a per-user access instance after onboarding prerequisites, but the repo-truth extraction marks this as PARTIAL because ONB reaches PH2 access storage directly. Future Access/Governance must own policy, templates, and scope.

Access and Voice ID are distinct. Voice ID is evidence. Repo evidence shows access step-up can consider verification/device posture, but no path was found where Voice ID alone grants access or authority.

Access and Link are distinct. PH1.LINK has link activation and role-proposal/dual-role-conflict draft surfaces, but raw links do not grant authority. Access stays with PH1.ACCESS/PH2.ACCESS and Access/Governance.

Active current product functions include access instance upsert, access overrides, access gate decisions, AP schema create/update/activate/retire, AP authoring review/channel/rule/confirmation, tenant overlays, board policy/vote rows, schema-chain reads, access instance compile lineage, simulation access checks for link/capability/access/reminder/calendar/BCAST policy paths, and app/client rendering of bounded access identifiers or authority-state posture.

Partial or unclear areas include workspace-specific scope, full company/entity access modeling, field-level private data policy, user-facing PH1.WRITE access-denial wording, end-to-end JD live access acceptance, SQL live persistence proof beyond migrations/tests, approval threshold resolution beyond vote row persistence, and old compatibility payroll/private/protected classification in Adapter.

Biggest risks/gaps:

- PRIVATE_DATA_READ_GATE_GAP: `sensitive_data_request` and `financial_auth` exist, but broad field/resource classification and product read paths are partial.
- WORKSPACE_SCOPE_GAP: tenant scope is strong; workspace scope is not proven as a first-class access dimension.
- ACCESS_WRITING_OWNER_RISK: denial and fail-closed wording appears hardcoded in several surfaces rather than PH1.WRITE-owned.
- ADAPTER_ACCESS_AUTHORITY_RISK: Adapter contains retained compatibility classification for protected/payroll flows. It fails closed but is wrong-owner semantic logic.
- ONB_ACCESS_GRANT_RISK: onboarding creates access instances after prerequisites; future governance must keep policy/template ownership canonical.

## 2. Repo Evidence Inventory

| Evidence Area | File / Path | Symbols / Routes / Tables | Status | Notes |
|---|---|---|---|---|
| Access kernel contract | `crates/selene_kernel_contracts/src/ph1access.rs` | `PH1ACCESS_CONTRACT_VERSION`, `ACCESS_AP_SCHEMA_CREATE_DRAFT`, `ACCESS_AP_SCHEMA_UPDATE_COMMIT`, `ACCESS_AP_SCHEMA_ACTIVATE_COMMIT`, `ACCESS_AP_SCHEMA_RETIRE_COMMIT`, `ACCESS_AP_OVERLAY_UPDATE_COMMIT`, `ACCESS_BOARD_POLICY_UPDATE_COMMIT`, `ACCESS_INSTANCE_COMPILE_COMMIT`, `AccessProfileScope`, `AccessProfileLifecycleState`, `AccessApprovalPolicySpec`, `AccessCompiledLineageRef` | FOUND | Contract shapes AP schema, overlays, approval policy, authoring review, lineage refs. |
| PH2 access kernel contract | `crates/selene_kernel_contracts/src/ph2access.rs` | none | NOT_FOUND | PH2 access is present through storage naming and DB wiring, not a separate kernel contract file. |
| Access engine runtime files | `crates/selene_engines/src/ph1access.rs`, `crates/selene_engines/src/ph2access.rs`, `crates/selene_os/src/ph1access.rs`, `crates/selene_os/src/ph2access.rs` | none | NOT_FOUND | Access is storage + OS gate wired, not standalone engine modules. |
| Policy contract | `crates/selene_kernel_contracts/src/ph1policy.rs` | `PolicyPromptDedupeDecideRequest`, `PolicyRulesetGetActiveRequest`, `PolicyPromptDecision` | FOUND | Prompt/ruleset policy owner, not access gate itself. |
| Governance contract | `crates/selene_kernel_contracts/src/ph1gov.rs` | `GovPolicyEvaluate`, `GovDecisionCompute`, `GovArtifactKind`, `GovDecisionStatus` | FOUND | Artifact governance owner, not per-user access truth. |
| Storage record truth | `crates/selene_storage/src/ph1f.rs` | `AccessInstanceRecord`, `AccessOverrideRecord`, `AccessGateDecisionRecord`, `AccessDecision`, `AccessEscalationTrigger`, `AccessApSchemaLedgerRecord`, `AccessOverlayRecord`, `AccessBoardPolicyRecord`, `AccessBoardVoteRecord` | FOUND | In-memory PH1.F implementation is the richest runtime evidence. |
| Storage repo trait | `crates/selene_storage/src/repo.rs` | `ph2access_upsert_instance_commit_row`, `ph2access_apply_override_commit_row`, `ph1access_gate_decide_row`, `ph1access_instance_compile_commit_row`, AP/overlay/board methods | FOUND | Defines durable storage interface names consumed by tests/runtime. |
| Access instance migration | `crates/selene_storage/migrations/0009_access_instance_tables.sql` | `access_instances`, `access_overrides`, tenant/user/idempotency indexes | FOUND | SQL truth for per-user access instance and override tables. |
| Master schema migration | `crates/selene_storage/migrations/0015_access_master_schema_tables.sql` | AP schemas ledger/current, overlays ledger/current, board policies/current/votes, compile lineage columns | FOUND | SQL truth for master access template schema. |
| AP authoring migration | `crates/selene_storage/migrations/0016_access_ap_authoring_review_tables.sql` | `access_ap_authoring_review_ledger`, `access_ap_authoring_review_current`, `access_ap_rule_review_actions_ledger` | FOUND | Explicit AP review-channel/rule-confirmation truth. |
| DB wiring | `docs/DB_WIRING/PH1_ACCESS_001_PH2_ACCESS_002.md` | `ACCESS_GATE_DECIDE_ROW`, `ACCESS_UPSERT_INSTANCE_COMMIT_ROW`, AP schema/overlay/board capabilities | FOUND | Canonical data ownership and failure-mode statement. |
| ECM | `docs/ECM/PH1_ACCESS_001_PH2_ACCESS_002.md` | Access capabilities, allowed callers `SELENE_OS_ONLY`, side-effect declarations | FOUND | Engine capability map confirms OS-only access calls. |
| Simulation catalog | `docs/08_SIMULATION_CATALOG.md` | `ACCESS_OVERRIDE_TEMP_GRANT_COMMIT`, `ACCESS_OVERRIDE_PERM_GRANT_COMMIT`, `ACCESS_OVERRIDE_REVOKE_COMMIT`, AP authoring/schema/overlay/board/compile simulations | FOUND | Access simulations are listed as DRAFT; current proof is not JD live. |
| Blueprint registry | `docs/09_BLUEPRINT_REGISTRY.md` | `ACCESS_SCHEMA_MANAGE`, `ACCESS_ESCALATION_VOTE`, `ACCESS_INSTANCE_COMPILE_REFRESH` | FOUND | Current access orchestration blueprints are registered. |
| DB ownership matrix | `docs/10_DB_OWNERSHIP_MATRIX.md` | `PH1.ACCESS.001_PH2.ACCESS.002` row | FOUND | Lists access gate and schema lifecycle storage truth. |
| Access blueprints | `docs/BLUEPRINTS/ACCESS_SCHEMA_MANAGE.md`, `ACCESS_ESCALATION_VOTE.md`, `ACCESS_INSTANCE_COMPILE_REFRESH.md` | Ordered PH1.C/PH1.NLP/PH1.X/PH1.ACCESS steps | FOUND | Blueprint-only orchestration; no implementation by itself. |
| Simulation executor | `crates/selene_os/src/simulation_executor.rs` | `enforce_access_gate`, `enforce_link_access_gate`, `enforce_link_delivery_access_gate`, `enforce_capreq_access_gate`, `enforce_access_schema_gate`, `enforce_access_escalation_vote_gate`, `enforce_access_instance_compile_gate`, `execute_access_step_up_dispatch_v1` | FOUND | Active OS gate enforcement for selected simulation candidate paths. |
| ONB access handoff | `crates/selene_os/src/ph1onb.rs`, `crates/selene_storage/src/ph1f.rs`, `crates/selene_kernel_contracts/src/ph1onb.rs` | `OnbAccessInstanceCreateCommitRequest`, `ph1onb_access_instance_create_commit`, `access_engine_instance_id`, `OnboardingStatus::AccessInstanceCreated` | FOUND/PARTIAL | ONB creates access instance after onboarding gates; policy ownership needs reconciliation. |
| LINK access context | `crates/selene_kernel_contracts/src/ph1link.rs`, `crates/selene_os/src/ph1link.rs` | `RoleProposalResult`, `DualRoleConflictEscalationResult`, `RoleProposeDraft`, `DualRoleConflictEscalateDraft` | PARTIAL | Link has role proposal/escalation drafts, not access grant. |
| App ingress | `crates/selene_os/src/app_ingress.rs` | `DispatchRequest::AccessStepUp`, `AppOnboardingContinueAction::AccessProvisionCommit`, authority state tests | PARTIAL | Ingress carries step-up/access-provision paths and fail-closed tests, but access owner remains storage/SimulationExecutor. |
| Adapter | `crates/selene_adapter/src/lib.rs` | `OnboardingContinueAdapterResponse.access_engine_instance_id`, `protected_fail_closed`, `payroll_private_business_read_intent`, `payroll_governed_business_intent` | PARTIAL/RISK | Adapter renders/bridges access identifiers and has retained protected/payroll compatibility classification. |
| Desktop client | `apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift` | `authorityStateCard`, `desktopOnboardingEntryCard`, `submitDesktopAccessProvisionCommit`, `accessEngineInstanceID` | PARTIAL | Desktop renders cloud-authored authority/access posture; it claims no local authority. |
| iPhone client | `apple/iphone/SeleneIPhone/SessionShellView.swift` | `ExplicitEntryContext.accessEngineInstanceID`, onboarding artifact/access summary | PARTIAL | iPhone renders access identifier/read-only onboarding state; no local access decision found. |
| Access storage tests | `crates/selene_storage/tests/ph1_access_ph2_access/db_wiring.rs` | `at_access_db_01` through `at_access_db_21` | FOUND | Tests tenant isolation, append-only, idempotency, gate decisions, AP activation, overlays, board votes, compile lineage. |
| Simulation executor tests | `crates/selene_os/src/simulation_executor.rs` | `at_sim_exec_19` through `at_sim_exec_24`, access step-up tests | FOUND | Tests access gate pass/fail/escalate for simulation dispatch and step-up audit. |
| Scripts | `scripts/check_agent_execution_core.sh`, `scripts/check_bcast_mhp_acceptance.sh`, builder permission scripts | PARTIAL | Some scripts check access-step-up branch or BCAST policy access tests; most builder permission scripts are adjacent, not Master Access. |
| Reports | `docs/reports/STAGE_8_FRESH_MEMORY_REAL_VOICE_PROOF.md`, `STAGE_8_5A...`, `STAGE_8_5C...`, voice evidence reports | PARTIAL | Evidence that protected payroll fails closed and Voice ID remains evidence-only; many are historical reports, not current engine code. |

## 3. Current Access Owner Map

| Responsibility | Current Owner / File | Correct Future Owner Hypothesis | Status | Notes |
|---|---|---|---|---|
| identity assertion | PH1.VOICE.ID/runtime governance and `identities` rows in PH1.F | Identity + Voice ID evidence owners | PARTIAL | Access instances FK to `identities.user_id`; Voice ID is evidence only. |
| user identity lookup | `Ph1fStore.identities`, `ph2access_get_instance_by_tenant_user` | Identity + Access | FOUND | Access gate requires matching user id on the instance. |
| tenant scope | `access_instances.tenant_id`, `access_scope_key`, tests | Access/Governance | FOUND | Tenant isolation is tested and enforced. |
| workspace scope | no first-class workspace field in access tables found | Workspace/Governance + Access | NOT_FOUND | DESIGN_GAP for workspace-specific permissions. |
| role template | `role_template_id` on access instance | Access/Governance role template registry | PARTIAL | Field exists; registry/semantic role hierarchy is not fully proven. |
| permission template | `baseline_permissions_json`, AP schema payloads, allow-list checks | Access template registry | PARTIAL | JSON allow-list and financial flag are simple; richer permission matrix missing. |
| per-user access instance | `access_instances`, `AccessInstanceRecord` | PH2.ACCESS | FOUND | Deterministic `accinst_` id by tenant/user. |
| access profile | AP schema ledger/current | PH1.ACCESS | FOUND | `access_profile_id`, scope, active schema version. |
| access grant | `ph2access_upsert_instance_commit`, `ph1access_instance_compile_commit`, overrides | Access/Governance | PARTIAL | Grant/update is a storage write; approval/policy semantics incomplete. |
| access revoke | `AccessOverrideType::Revoke`, suspended lifecycle through upsert | Access/Governance | PARTIAL | Revoke override exists; direct lifecycle revoke/expiration state not fully modeled. |
| access update | upsert/compile and overlays/AP schema lifecycle | Access/Governance | FOUND/PARTIAL | State exists but future policy/template registry needed. |
| access deny | `AccessDecision::Deny`, gate reason codes | PH1.ACCESS | FOUND | Missing instance, scope mismatch, suspended, disallowed action, sensitive deny. |
| access escalation | `AccessDecision::Escalate`, `AccessEscalationTrigger` | Access/Governance + AP approval | FOUND/PARTIAL | Step-up and AP approval triggers exist; full case lifecycle missing. |
| approval / AP | AP authoring review, board policy/vote, overrides | Access/Governance | PARTIAL | Vote rows exist; threshold resolution is blueprint/design more than runtime proof. |
| admin/manager/HR/owner permissions | `role_template_is_owner_or_admin` helper, role string tests | Access role/template registry | PARTIAL/RISK | Owner/admin shortcut uses string contains in SimulationExecutor. |
| company/customer/supplier/employee/friend access types | role strings, ONB invitee type, docs | Access/Governance | PARTIAL | Product taxonomy not normalized. |
| private data read gate | `sensitive_data_request`, `financial_auth`, Adapter payroll read detection | Access + PH1.X + PH1.WRITE | PARTIAL | Strong gap beyond payroll compatibility. |
| protected execution gate | SimulationExecutor access gates plus simulation catalog | PH1.X + Access + Authority + SimulationExecutor | FOUND/PARTIAL | Access checked before some simulations; full protected execution stack remains broader. |
| onboarding access handoff | PH1.ONB `AccessInstanceCreateCommit` | PH1.ONB coordinates, Access/Governance owns policy | PARTIAL/RISK | Current ONB writes access instance after prerequisites. |
| link access handoff | PH1.LINK role proposal/dual role conflict drafts | PH1.LINK proposes; Access/Governance decides | PARTIAL | Link does not grant access. |
| voice identity evidence handoff | access verification/device trust levels; step-up dispatch | Voice ID/Human Presence evidence into Access | PARTIAL | No direct Voice ID to access grant found. |
| device trust handoff | `AccessDeviceTrustLevel`, step-up device trust check | Device/Human Presence + Access | FOUND/PARTIAL | DTL1-DTL4 exist; deeper device trust provenance incomplete. |
| access audit/provenance | append-only ledgers, audit for access step-up | PH1.J + Access | PARTIAL | Many writes are ledger rows; explicit PH1.J audit only proven for capreq step-up. |
| Desktop rendering | Swift read-only authority/access posture | Desktop render only | PARTIAL | No local access decision found. |
| iPhone rendering | Swift explicit entry/onboarding artifact access preview | iPhone render only | PARTIAL | No local access decision found. |
| Adapter transport | Adapter response structs and compatibility logic | Adapter transport only | PARTIAL/RISK | Contains wrong-owner semantic compatibility classifications. |
| old compatibility paths | Adapter payroll/protected phrase helpers; link role proposal drafts | Canonical PH1.X/Access/Governance future owners | PARTIAL/RISK | Must not delete before active-caller proof. |

## 4. Current Access Lifecycle

### A. Access request / gate lifecycle

Stage: request reaches simulation candidate dispatch.

- Owner: Selene OS / SimulationExecutor.
- Symbols/files: `execute_ph1x_dispatch_simulation_candidate`, `enforce_access_gate`, `enforce_link_access_gate`, `enforce_capreq_access_gate`, `enforce_access_schema_gate`.
- Inputs: `actor_user_id`, tenant id, requested action string such as `LINK_INVITE`, `DELIVERY_SEND`, `CAPREQ_MANAGE`, `ACCESS_SCHEMA_MANAGE`, `REMINDER_SET`, `CALENDAR_EVENT_CREATE`.
- Outputs: `Ok(())`, `SimulationDispatchOutcome::AccessGatePassed`, or fail-closed `StorageError::ContractViolation`.
- State changes: none in the gate itself.
- Audit evidence: access step-up path records PH1.ACCESS/CAPREQ audit; plain gate reads are not generally stored as rows.
- Gaps: no universal access gate for every possible private/protected product function is proven.

Stage: access instance lookup.

- Owner: PH2.ACCESS storage.
- Symbols/files: `ph2access_get_instance_by_tenant_user`, `ph2access_get_instance_by_id`.
- Inputs: tenant id, user id, access instance id.
- Outputs: `AccessInstanceRecord` or missing instance.
- State changes: none.
- Audit evidence: not directly audited in the getter.
- Gaps: workspace/resource scope not first-class.

Stage: deterministic gate decision.

- Owner: PH1.ACCESS storage decision logic.
- Symbols/files: `ph1access_gate_decide`.
- Inputs: `user_id`, `access_engine_instance_id`, `requested_action`, `access_request_context`, `device_trust_level`, `sensitive_data_request`, `now`.
- Outputs: `AccessGateDecisionRecord` with `Allow`, `Deny`, or `Escalate`.
- State changes: none.
- Audit evidence: DB wiring says gate reads emit audit only in explicit enforcement traces; current generic gate read audit not found.
- Gaps: no full PH1.WRITE denial owner; no field-level access map beyond simple JSON flags.

Stage: gate failure or escalation.

- Owner: SimulationExecutor for dispatch refusal, PH1.ACCESS for reason code.
- Symbols/files: `fail_closed_dispatch_access_error`, `AccessEscalationTrigger::StepUpProofRequired`, `AccessEscalationTrigger::ApApprovalRequired`.
- Inputs: gate decision.
- Outputs: `ACCESS_SCOPE_VIOLATION` or `ACCESS_AP_REQUIRED`.
- State changes: none unless routed to step-up audit.
- Audit evidence: step-up audit emits START/FINISH rows.
- Gaps: full approval case state machine not implemented as a distinct current table beyond board votes/overrides.

### B. Access instance lifecycle

Stage: per-user access instance upsert.

- Owner: PH2.ACCESS storage.
- Symbols/files: `ph2access_upsert_instance_commit`, `access_instances`.
- Inputs: tenant id, user id, role template id, access mode, permissions JSON, identity verified flag, verification level, device trust, lifecycle state, policy snapshot ref, idempotency key.
- Outputs: `AccessInstanceRecord`.
- State changes: upsert `access_instances`.
- Audit evidence: storage row only; PH1.J event not found for generic upsert.
- Gaps: role template registry and policy snapshot semantics are partial.

Stage: access instance compile.

- Owner: PH1.ACCESS/PH2.ACCESS storage.
- Symbols/files: `ph1access_instance_compile_commit`, `AccessCompiledLineageRef`, AP current tables, overlay current tables, `positions`.
- Inputs: active global AP, optional tenant AP, overlays, optional position id, permissions JSON, access mode, identity/device fields.
- Outputs: `AccessInstanceRecord` with compiled lineage refs.
- State changes: access instance upsert with lineage fields.
- Audit evidence: storage row and tests; explicit audit event not found.
- Gaps: compile does not prove a full enterprise template registry or workspace scope.

Stage: override application.

- Owner: PH2.ACCESS storage.
- Symbols/files: `ph2access_apply_override_commit`, `access_overrides`, `AccessOverrideType`.
- Inputs: override type, scope JSON, approver user id, simulation id, reason, start/end, idempotency key.
- Outputs: `AccessOverrideRecord`.
- State changes: append override row.
- Audit evidence: append-only storage row; explicit PH1.J event not found.
- Gaps: approval threshold satisfaction is not fully enforced in this method; caller must gate.

### C. AP schema / template lifecycle

Stage: AP authoring review channel.

- Owner: PH1.ACCESS.
- Symbols/files: `ph1access_ap_authoring_review_channel_commit`, `access_ap_authoring_review_ledger/current`.
- Inputs: tenant/scope, access profile id, schema version id, channel `PHONE_DESKTOP` or `READ_OUT_LOUD`.
- Outputs: `AccessApAuthoringReviewCurrentRecord`.
- State changes: append review ledger and update current.
- Audit evidence: ledger rows.
- Gaps: no PH1.WRITE review UX proof.

Stage: AP rule action.

- Owner: PH1.ACCESS.
- Symbols/files: `ph1access_ap_authoring_rule_action_commit`, `AccessApRuleReviewActionPayload`.
- Inputs: action `AGREE`, `DISAGREE`, `EDIT`, `DELETE`, `DISABLE`, `ADD_CUSTOM_RULE` plus bounded refs.
- Outputs: `AccessApRuleReviewActionRecord`.
- State changes: append rule review action row.
- Audit evidence: ledger rows.
- Gaps: rule semantics inside payload JSON are not fully interpreted.

Stage: AP confirmation.

- Owner: PH1.ACCESS.
- Symbols/files: `ph1access_ap_authoring_confirm_commit`.
- Inputs: confirmation state `PENDING_ACTIVATION_CONFIRMATION`, `CONFIRMED_FOR_ACTIVATION`, or `DECLINED`.
- Outputs: current confirmation state.
- State changes: append confirmation ledger and update current.
- Audit evidence: ledger rows.
- Gaps: no live human confirmation UI proof.

Stage: AP schema create/update/activate/retire.

- Owner: PH1.ACCESS.
- Symbols/files: `ph1access_ap_schema_lifecycle_commit`, `access_ap_schemas_ledger/current`.
- Inputs: scope, profile id, schema version id, event action, payload JSON, reason, creator, idempotency.
- Outputs: `AccessApSchemaLedgerRecord`.
- State changes: append ledger; activate/retire updates current projection.
- Audit evidence: ledger rows with activation review lineage.
- Gaps: no complete schema diff/review owner; simulations are DRAFT in catalog.

### D. Board approval / escalation lifecycle

Stage: board policy update.

- Owner: PH1.ACCESS.
- Symbols/files: `ph1access_board_policy_update_commit`, `access_board_policy_ledger/current`.
- Inputs: tenant, board policy id, policy version, payload JSON, event action.
- Outputs: `AccessBoardPolicyRecord`.
- State changes: append policy ledger and update current on activate/retire.
- Audit evidence: ledger rows.
- Gaps: policy payload validation is bounded but not semantically complete.

Stage: board vote.

- Owner: PH1.ACCESS.
- Symbols/files: `ph1access_board_vote_commit`, `access_board_votes_ledger`.
- Inputs: tenant, escalation case id, board policy id, voter user id, vote value, reason, idempotency.
- Outputs: `AccessBoardVoteRecord`.
- State changes: append vote row.
- Audit evidence: vote ledger.
- Gaps: threshold resolution and override application after threshold appear blueprint-level, not fully implemented as a case resolver.

### E. Onboarding-created access instance

- Owner: PH1.ONB currently coordinates, PH2.ACCESS storage persists.
- Symbols/files: `ph1onb_access_instance_create_commit`, `OnbAccessInstanceCreateCommitRequest`.
- Inputs: onboarding session id, user id, tenant id, role id, idempotency key.
- Outputs: `OnbAccessInstanceCreateResult.access_engine_instance_id`.
- State changes: creates/upserts access instance; session becomes `AccessInstanceCreated`.
- Audit evidence: onboarding session state and storage row.
- Gaps: ONB direct access storage call is a future owner-boundary reconciliation point.

### F. Private read / protected precheck

- Owner: PARTIAL across PH1.X/Adapter/SimulationExecutor/Access.
- Symbols/files: `sensitive_data_request`, `financial_auth`, Adapter payroll helpers, protected fail-closed reports.
- Inputs: action/read request, protected risk, private/sensitive flag.
- Outputs: deny/fail-closed or public answer classification.
- State changes: none.
- Audit evidence: adapter/internal history refs and reports; generic access audit partial.
- Gaps: broad private data read gate is not complete.

## 5. Data Model / Contracts / Packets

### Request / response / contract structures

| Name | File | Kind | Status | Notes |
|---|---|---|---|---|
| `AccessProfileId` | `ph1access.rs` | ID wrapper | FOUND | AP/profile identifier. |
| `AccessOverlayId` | `ph1access.rs` | ID wrapper | FOUND | Overlay identifier. |
| `BoardId` | `ph1access.rs` | ID wrapper | FOUND | Board identifier. |
| `AccessProfileSchemaRecord` | `ph1access.rs` | contract record | FOUND | Profile schema metadata with lifecycle, scope, owner tenant, allow list. |
| `AccessOverlayOpSpec` | `ph1access.rs` | contract record | FOUND | Overlay op payload; op types include add/remove/tighten/escalation policy. |
| `AccessApprovalPolicySpec` | `ph1access.rs` | contract record | FOUND | Single approver, N-of-M, board quorum, unanimous board, mixed. |
| `AccessCompiledLineageRef` | `ph1access.rs` | contract record | FOUND | Global/tenant AP refs, overlay refs, optional position id. |
| `AccessApRuleReviewActionPayload` | `ph1access.rs` | contract payload | FOUND | Rule review action with suggested rule/capability/constraint/escalation refs. |
| `AccessApAuthoringReviewState` | `ph1access.rs` | contract state | FOUND | Review channel and confirmation state for AP authoring. |
| `AccessGateDecisionRecord` | `ph1f.rs` | response/equivalent packet | EQUIVALENT_FOUND | Gate output with decision, mode, flags, escalation trigger, reason. |
| `AccessSchemaChainReadResult` | `ph1f.rs` | response/equivalent packet | EQUIVALENT_FOUND | Active global/tenant AP, overlays, board policy. |

### Records / table-backed structures

| Name | File / Table | Status | Fields / Notes |
|---|---|---|---|
| `AccessInstanceRecord` | `ph1f.rs`, `access_instances` | FOUND | Tenant/user, role template, mode, permissions JSON, identity/device posture, lifecycle, policy snapshot, compile lineage, timestamps, idempotency. |
| `AccessOverrideRecord` | `ph1f.rs`, `access_overrides` | FOUND | Override id/type/status, scope JSON, approver, simulation id, reason, time window, idempotency. |
| `AccessApSchemaLedgerRecord` | `ph1f.rs`, `access_ap_schemas_ledger` | FOUND | Scope/profile/version/action/lifecycle/payload/reason/creator/activation review lineage. |
| `AccessApSchemaCurrentRecord` | `ph1f.rs`, `access_ap_schemas_current` | FOUND | Current active AP version per scope/profile. |
| `AccessApAuthoringReviewLedgerRecord` | `ph1f.rs`, `access_ap_authoring_review_ledger` | FOUND | Channel/confirmation events for AP review. |
| `AccessApAuthoringReviewCurrentRecord` | `ph1f.rs`, `access_ap_authoring_review_current` | FOUND | Current review channel and confirmation state. |
| `AccessApRuleReviewActionRecord` | `ph1f.rs`, `access_ap_rule_review_actions_ledger` | FOUND | Rule action rows with bounded refs. |
| `AccessOverlayRecord` | `ph1f.rs`, `access_ap_overlay_ledger` | FOUND | Tenant overlay lifecycle payload. |
| `AccessOverlayCurrentRecord` | `ph1f.rs`, `access_ap_overlay_current` | FOUND | Active overlay version. |
| `AccessBoardPolicyRecord` | `ph1f.rs`, `access_board_policy_ledger` | FOUND | Tenant board policy lifecycle payload. |
| `AccessBoardPolicyCurrentRecord` | `ph1f.rs`, `access_board_policy_current` | FOUND | Active board policy version. |
| `AccessBoardVoteRecord` | `ph1f.rs`, `access_board_votes_ledger` | FOUND | Escalation case vote row. |

### Enums and status states

| Name | Values | Status | Notes |
|---|---|---|---|
| `AccessMode` | `R`, `W`, `A`, `X` | FOUND | Ranked read/write/approve/execute style modes. |
| `AccessVerificationLevel` | `None`, `PasscodeTime`, `Biometric`, `StepUp` | FOUND | Used in access instance and step-up. |
| `AccessDeviceTrustLevel` | `Dtl1`, `Dtl2`, `Dtl3`, `Dtl4` | FOUND | Lower trust can escalate action mode A+. |
| `AccessLifecycleState` | `Restricted`, `Active`, `Suspended` | FOUND | Gate escalates restricted/unverified, denies suspended. |
| `AccessOverrideType` | `OneShot`, `Temporary`, `Permanent`, `Revoke` | FOUND | Revoke persists as revoked override row. |
| `AccessOverrideStatus` | `Active`, `Expired`, `Revoked` | FOUND | Derived at write time. |
| `AccessDecision` | `Allow`, `Deny`, `Escalate` | FOUND | Core gate decision. |
| `AccessEscalationTrigger` | `StepUpProofRequired`, `ApApprovalRequired` | FOUND | Gate escalation trigger. |
| `AccessSchemaScope` | `Global`, `Tenant` | FOUND | No workspace scope found. |
| `AccessSchemaEventAction` | `CreateDraft`, `UpdateDraft`, `Activate`, `Retire` | FOUND | AP/overlay/board lifecycle action. |
| `AccessSchemaLifecycleState` | `Draft`, `Active`, `Retired` | FOUND | Lifecycle projection state. |
| `AccessApReviewChannel` | `PhoneDesktop`, `ReadOutLoud` | FOUND | AP authoring review mode. |
| `AccessApAuthoringConfirmationState` | `NeedsChannelChoice`, `ReviewInProgress`, `PendingActivationConfirmation`, `ConfirmedForActivation`, `Declined` | FOUND | Some states contract-defined; storage confirm accepts pending/confirmed/declined. |
| `AccessBoardVoteValue` | `Approve`, `Reject` | FOUND | Vote values. |

### Error / reason code equivalents

| Reason / Error | Source | Status | Notes |
|---|---|---|---|
| `ACCESS_REASON_ALLOWED` | `ph1f.rs` | FOUND | Internal numeric reason. |
| `ACCESS_REASON_DENIED` | `ph1f.rs` | FOUND | Used for suspended/disallowed. |
| `ACCESS_REASON_ESCALATE_REQUIRED` | `ph1f.rs` | FOUND | Restricted/unverified escalation. |
| `ACCESS_REASON_INSTANCE_MISSING` | `ph1f.rs` | FOUND | Missing access instance fail closed. |
| `ACCESS_REASON_SCOPE_MISMATCH` | `ph1f.rs` | FOUND | User scope mismatch. |
| `ACCESS_REASON_AP_REQUIRED` | `ph1f.rs` | FOUND | Mode upgrade approval required. |
| `ACCESS_REASON_SENSITIVE_DENY` | `ph1f.rs` | FOUND | Sensitive data request without financial auth. |
| `ACCESS_REASON_DEVICE_UNTRUSTED` | `ph1f.rs` | FOUND | Device trust too low for A+ context. |
| `ACCESS_SCOPE_VIOLATION` | SimulationExecutor | FOUND | Fail-closed dispatch reason for deny. |
| `ACCESS_AP_REQUIRED` | SimulationExecutor | FOUND | Fail-closed dispatch reason for escalation. |
| `StorageError::ContractViolation` / `ForeignKeyViolation` / `AppendOnlyViolation` | `ph1f.rs`, tests | FOUND | Storage validation and append-only guarantees. |

## 6. Access Types And Product Functions

| Access/Product Type | Evidence Found | Current Behavior | Owner | Security Implications | Missing Design |
|---|---|---|---|---|---|
| personal user access | ONB creates `personal_` tenant fallback when no tenant | Creates scoped per-user access instance | PH1.ONB + PH2.ACCESS | Could blur personal vs tenant policy | Personal access policy registry missing. |
| employee access | ONB `InviteeType::Employee`, position prereq, role ids | Employee onboarding can create access instance after prerequisites | PH1.ONB + PH2.ACCESS | HR/access risk | Access template integration partial. |
| customer access | docs/search found no concrete customer access template | UNKNOWN | Access/Governance | Customer data exposure risk | DESIGN_GAP. |
| supplier access | docs/search found no concrete supplier access template | UNKNOWN | Access/Governance | Bank/compliance data exposure risk | DESIGN_GAP. |
| contractor access | role template field can carry arbitrary role, but no contractor model found | PARTIAL | Access/Governance | Contract/payment/system access risk | DESIGN_GAP. |
| friend/family/personal connection access | no dedicated access type found | NOT_FOUND | Access/Governance/Personal scope | Could overgrant if generic | DESIGN_GAP. |
| tenant admin access | tenant-scoped access instance, owner/admin helper | PARTIAL | Access/Governance | Admin strings influence gates | Role registry needed. |
| workspace access | no access table workspace id found | NOT_FOUND | Workspace/Governance + Access | Cross-workspace exposure risk | WORKSPACE_SCOPE_GAP. |
| company admin access | tenant/company tables and positions exist, but access templates are partial | PARTIAL | Company/Tenant/Governance | Company entity scope unclear | DESIGN_GAP. |
| HR access | no explicit HR role registry found | PARTIAL/UNKNOWN | Access/Governance | HR data sensitive | Field-level access missing. |
| payroll access | Adapter payroll private/protected classification, `financial_auth` flag | PARTIAL/RISK | Access + Payroll owner | Salary/payroll exposure risk | Strong private read gate needed. |
| finance access | `financial_auth` flag only | PARTIAL | Access/Governance | Finance data sensitive | Finance permission matrix missing. |
| roster/scheduler access | SimulationExecutor calendar/reminder gates; no roster access model found | PARTIAL | Scheduler/Roster + Access | Schedule mutation risk | Boundary map needed. |
| inventory access | no explicit inventory permission found | NOT_FOUND | Inventory + Access | Resource exposure risk | DESIGN_GAP. |
| customer data access | no explicit field/resource model found | NOT_FOUND | CRM/Customer + Access | Customer privacy risk | DESIGN_GAP. |
| supplier data access | no explicit field/resource model found | NOT_FOUND | Supplier + Access | Supplier bank/privacy risk | DESIGN_GAP. |
| onboarding access | `ONB_ACCESS_INSTANCE_CREATE_COMMIT`, access instance create | FOUND/PARTIAL | PH1.ONB coordinates, Access owns policy | Onboarding could create wrong role | Access handoff proof needed. |
| voice/device access posture | verification/device trust fields, step-up dispatch | PARTIAL | Voice ID/Device evidence + Access | Evidence must not equal authority | Strong evidence mapping needed. |
| protected action authority | SimulationExecutor access gates + simulation guard | PARTIAL | PH1.X + Authority + Simulation | Access is not enough to execute | Full authority/proof stack needed. |
| read-only access | `AccessMode::R` | FOUND | Access | Lower risk, still scoped | Resource-level reads incomplete. |
| write access | `AccessMode::W` | FOUND | Access | Mutations need simulation | W action semantics not exhaustive. |
| approval access | `AccessMode::A`, board policies/votes | FOUND/PARTIAL | Access/Governance | Approval abuse risk | Threshold resolver missing. |
| emergency/break-glass access | no clear break-glass model found | NOT_FOUND | Access/Governance/Authority | High risk | SECURITY_GAP. |

## 7. Role / Permission / Access Template Logic

Access templates exist as AP schemas at global and tenant scope. Evidence: `AccessProfileSchemaRecord`, `access_ap_schemas_ledger/current`, `AccessProfileVersionRef`, `AccessCompiledLineageRef`, and access blueprints.

Role templates exist as `role_template_id` strings on `access_instances`. Current runtime also has `role_to_default_access_mode`, which infers mode from role text containing `admin`, `approve`, `write`, or `editor`. SimulationExecutor has `role_template_is_owner_or_admin`, which checks whether the string contains `owner` or `admin`.

Permissions are partly explicit and partly implied:

- Explicit allow-list: `baseline_permissions_json` can contain `"allow":["ACTION"]` or `"allow":["*"]`.
- Sensitive flag: `baseline_permissions_json` can contain `"financial_auth":true`.
- Legacy behavior: if no `"allow":[` appears in baseline permissions JSON, `allows_requested_action` returns true to preserve backwards compatibility.
- Implied mode: `AccessMode` ranks R/W/A/X and can be raised by active overrides that include `grant_mode`.

Permissions attach currently to per-user access instances through `baseline_permissions_json`, role template id, effective access mode, and active overrides. AP schemas and overlays provide source lineage for compiled instances. Tenant scope is explicit. Workspace/resource/field-level attachment is not fully proven.

A master access template model is PARTIAL. AP schema ledger/current, overlays, and compile lineage are real, but the semantic role/permission registry and review UX are not fully implemented.

Per-user access instance is FOUND. `AccessInstanceRecord` is persisted and tests prove idempotent upsert by tenant/user/idempotency.

Onboarding creates access through `ph1onb_access_instance_create_commit` after terms, primary device confirmation, persona lock, sender verification if required, and employee position prerequisites. This is current repo truth but remains OWNER_GAP for future Access/Governance policy ownership.

Link activation does not create access in the evidence inspected. PH1.LINK can create role-proposal and dual-role conflict draft rows, but no raw link access grant was found.

Employee/customer/supplier/friend roles are not fully differentiated in Access. Employee appears through ONB and position integration. Customer/supplier/friend access taxonomy is DESIGN_GAP.

High-access roles like CEO/CFO/admin/HR are not first-class semantic roles in Access. The code has generic role strings, AP profiles such as `ap_ceo` in tests, board/quorum policy rows, and `owner/admin` string helpers. This is PARTIAL and high risk.

Role hierarchy is not fully implemented. `AccessMode` rank gives a simple mode hierarchy, and AP/overlay/board lineage gives template composition, but not a complete enterprise role hierarchy.

Approval/escalation path exists partially through `AccessDecision::Escalate`, `AccessEscalationTrigger::ApApprovalRequired`, board policy/vote rows, and override commits. Full escalation case resolution and threshold enforcement are not fully proven.

## 8. Tenant / Workspace / Company Scope

Tenant scope is enforced.

Evidence:

- `access_instances` includes `tenant_id`.
- `access_overrides` requires tenant id and checks it matches the access instance tenant.
- AP schema scope uses `GLOBAL` or tenant id through `access_scope_key`.
- `ph1access_read_schema_chain` requires tenant id and fails when tenant overlays/board policies are not in that tenant.
- `at_access_db_01_tenant_isolation_enforced` and `at_access_db_16_tenant_isolation_enforced_for_schema_chain_reads` test tenant isolation.

Workspace scope is NOT_FOUND as a first-class field in current access tables/records. Workspace references exist in higher architecture and other product domains, but no concrete `workspace_id` access dimension was found in the inspected access storage/model. Future workspace access must be reconciled.

Company/entity scope is PARTIAL. Tenant company and position records exist (`TenantCompanyRecord`, `PositionRecord`), and access instance compile can validate a position id. However, company-specific access profiles and company admin access are not fully modeled in Access.

Multiple tenants/workspaces per user are PARTIAL. The per-user unique key is `(tenant_id, user_id)`, so a user can have one access instance per tenant. Multi-workspace access inside a tenant is not proven.

Roles scoped to one workspace are NOT_FOUND.

Country/region/department variation is PARTIAL. Position records include jurisdiction and schedule type, but access policy overlays by country/department are not fully proven as Access behavior.

Tenant mismatch fails closed through override validation, schema-chain reads, and SimulationExecutor missing instance/tenant access gate.

Workspace mismatch behavior is UNKNOWN because workspace-scoped access was not found.

## 9. Private Data Read Gate

Current private data read support is PARTIAL.

Repo evidence:

- `ph1access_gate_decide` accepts `sensitive_data_request: bool`.
- If `sensitive_data_request` is true and `baseline_permissions_json` lacks `"financial_auth":true`, the gate denies with `SENSITIVE_FIELDS_BLOCKED` and `ACCESS_REASON_SENSITIVE_DENY`.
- Adapter has `payroll_private_business_read_intent` and `payroll_governed_business_intent` compatibility helpers.
- Stage reports show public payroll knowledge separated from private/protected payroll requests and protected execution fails closed.

Answers to required questions:

- Is private data read separated from normal chat? PARTIAL. Adapter/payroll compatibility separates some requests, but a canonical PH1.X + Access field-level read gate is not fully proven.
- What access gate applies? `ph1access_gate_decide(..., sensitive_data_request=true)` can deny without `financial_auth`.
- What owner validates scope? Access validates simple financial flag and user/instance scope. Field/resource owner validation is DESIGN_GAP.
- Are reads audited? AUDIT_GAP. Generic private read access audit was not found.
- Are fields classified? PARTIAL. Financial flag exists; no complete field classification registry found.
- Does PH1.X route private read separately from protected execution? PARTIAL. PH1.X architecture and Adapter reports point that way; current full canonical route is not proven.
- Does PH1.WRITE phrase denial/explanation? NOT_FOUND. Denial/fail-closed strings appear hardcoded in Adapter/OS surfaces.

Examples:

- `Show Tim's salary`: likely private HR/payroll read. Current canonical field-level gate is DESIGN_GAP.
- `What was our gross margin last month?`: company private/finance read. Current financial authorization flag may be relevant, but exact product owner/data source gate is NOT_FOUND.
- `Show customer invoice`: customer data read. Current customer field-level permission NOT_FOUND.
- `Show supplier bank details`: supplier/private finance read. Current supplier/bank field permission NOT_FOUND.

## 10. Protected Execution Gate

Current protected execution gating is PARTIAL and split.

Access checks:

- SimulationExecutor runs access gates before selected commit candidates.
- Access gate checks instance existence, user match, lifecycle, identity verified, allow-list, sensitive flag, device trust, and mode.
- Gate returns `ALLOW`, `DENY`, or `ESCALATE`.

Authority checks:

- Architecture and runtime governance docs separate authority from access.
- App/client surfaces render cloud-authored authority posture.
- Access itself does not decide full protected authority.
- Full authority state completion is outside the access extraction.

SimulationExecutor checks:

- Maps intent drafts to simulation IDs.
- Requires simulation catalog active/valid before dispatch.
- Dispatch branch enforces access before link, link delivery, capability request, access schema management, access escalation voting, access instance compile, calendar event creation, reminder mutation, and BCAST policy update paths.

Required before execution:

- Valid intent draft fields.
- Tenant resolved.
- Access gate `ALLOW`.
- Simulation id valid and active.
- For access schema management, required AP review/channel/rule fields.
- For high-risk paths, confirmation/approval is blueprint-required but not always fully proven in runtime.

Access does not execute protected actions directly. It returns gate decisions or writes access-owned records.

LLM/OpenAI execution from access was NOT_FOUND.

Confirmation is PARTIAL. Blueprints require PH1.X confirmation for access schema/vote/compile. Storage AP activation requires confirmed authoring review when review exists. Live end-to-end user confirmation is not fully proven.

Audit logs are PARTIAL. Access storage ledgers are durable evidence; PH1.J-style audit is visible for access step-up. Generic access gates and private reads need stronger audit proof.

Fail-closed behavior is implemented in several places:

- Missing instance -> `Deny`.
- User mismatch -> `Deny`.
- Suspended instance -> `Deny`.
- Restricted/unverified -> `Escalate`.
- Disallowed action -> `Deny`.
- Sensitive without financial auth -> `Deny`.
- Low device trust for A+ -> `Escalate`.
- Requested mode above effective mode -> `Escalate`.
- SimulationExecutor converts deny/escalate to no side-effect errors.

## 11. Interaction With PH1.ONB

Onboarding/access interaction is FOUND and PARTIAL.

Evidence:

- `OnbAccessInstanceCreateCommitRequest` includes onboarding session id, user id, optional tenant id, role id, and idempotency key.
- `ph1onb_access_instance_create_commit` enforces terms accepted, primary device confirmed, EMO/persona lock audit event, sender verification when schema-required, and employee position prerequisites.
- It creates an identity row if missing.
- It calls `ph2access_upsert_instance_commit` with `role_to_default_access_mode(&role_id)`, baseline permissions `{"financial_auth":false}`, identity verified true, `PasscodeTime`, `Dtl3`, lifecycle `Active`, and policy snapshot `role_template:{role_id}`.
- It writes `access_engine_instance_id` back to onboarding session and sets status `AccessInstanceCreated`.

Does onboarding create a per-user access instance? Yes, after prerequisites.

What fields does ONB send to Access? Tenant id, user id, role id, derived access mode, baseline permissions JSON, verification/device posture, lifecycle, policy snapshot, idempotency.

Does ONB select role/access templates? PARTIAL. It passes a role id. Full access template selection is not proven.

Does ONB call PH2 access storage directly? Yes, inside PH1.F storage method `ph1onb_access_instance_create_commit`.

Is that correct or owner-gap? Current repo truth accepts it as active; future architecture should treat it as OWNER_GAP/PARTIAL because Access/Governance must own policy/template/scope.

Risks for employee/executive onboarding:

- Role id string can shape default access mode.
- High-risk executive roles are not semantically isolated.
- ONB access creation currently uses generic baseline permissions.
- Salary/payroll/finance fields and access templates require future governance.

## 12. Interaction With PH1.LINK

PH1.LINK interaction is PARTIAL.

Does a link grant access? No direct evidence that link activation grants access.

Does link activation create access? NOT_FOUND.

Does link carry access context? PARTIAL. Link prefilled context can include tenant id, and link role-proposal/dual-role-conflict structures exist. The actual access instance is created in ONB, not LINK.

Does link only start onboarding? For invite/onboarding flows, link activation hands context to onboarding; PH1.LINK stops before onboarding/access completion.

Does role proposal link exist? There is `RoleProposalResult`, `RoleProposeDraftRequest`, and role-proposal runtime in PH1.LINK. It appears to draft/propose, not grant.

Does PH1.LINK have role/access proposal behavior? Yes, PARTIAL. Role proposal and dual role conflict escalation draft behavior exists.

What remains Access/Governance-owned? Role template approval, permission grant, per-user access instance, escalation approval, access revocation/update, private/protected gate decisions.

Critical rule: raw link text must never grant authority.

## 13. Interaction With Voice Identity + Human Presence

Voice/device/access interaction is PARTIAL.

Evidence:

- `AccessVerificationLevel` has `PasscodeTime`, `Biometric`, `StepUp`.
- `AccessDeviceTrustLevel` has `Dtl1` through `Dtl4`.
- SimulationExecutor `execute_access_step_up_dispatch_v1` evaluates access gate and challenge method.
- `step_up_challenge_satisfied` maps verification level to passcode/biometric outcomes.
- Voice ID reports state evidence-only posture and no authority grant.

Does Voice ID affect access? PARTIAL. Voice identity itself was not found as a direct access grant. Verification/device trust may affect step-up.

Does voice enrollment grant access? No evidence found.

Does device proof affect access? Yes, via device trust level and onboarding primary device confirmation before access instance creation.

Does speaker confidence affect private data reads? NOT_FOUND.

Does Voice ID feed Access as evidence only? Current architecture says yes; repo evidence does not show Voice ID as authority.

Missing:

- Formal Voice ID evidence to Access decision packet.
- Speaker confidence/private read policy.
- Human presence binding for step-up beyond device/verification enums.

Critical rule: Voice ID is evidence only. Voice ID must not grant authority.

## 14. Interaction With PH1.X / Request Decision Lattice

PH1.X interaction is PARTIAL.

Evidence:

- `IntentType::AccessSchemaManage`, `AccessEscalationVote`, and `AccessInstanceCompileRefresh` exist in simulation dispatch.
- `DispatchRequest::SimulationCandidate` and `DispatchRequest::AccessStepUp` are handled by app ingress / SimulationExecutor.
- Blueprints route access flows through PH1.C, PH1.NLP, PH1.X clarification/confirmation, then Access gate.
- Global Request Decision Lattice design says private/protected requests need deterministic validation.

Does PH1.X classify public/private/protected requests? PARTIAL. Architecture and adapter reports show intent; canonical current runtime has selected intent drafts and protected fail-closed behavior, but old Adapter compatibility classification remains.

Does PH1.X call Access? In current runtime, SimulationExecutor consumes PH1.X dispatch/candidate outputs and calls Access gate. Direct PH1.X-to-Access implementation is PARTIAL.

Does Access feed PH1.X? Gate results become dispatch outcomes/errors and can feed response, but no full PH1.X access decision packet was found.

What currently decides lane/risk? Mix of IntentDraft intent type, SimulationExecutor guards, and Adapter compatibility classifiers for public/private/protected payroll. This needs reconciliation.

Does Access know about ambiguous requests? No. Access receives resolved requested action/mode/sensitive flag only.

Does Access fail closed on uncertainty? It fails closed on missing instance/scope/disallowed/sensitive/device/mode issues. Ambiguity is a PH1.X/PH1.N/PH1.WRITE responsibility.

Future reconciliation needed:

- PH1.X should classify risk and required gate.
- Access should decide scope/permission only.
- PH1.WRITE should explain denial/escalation.
- Adapter phrase logic should retire after canonical PH1.X owner proof.

## 15. Interaction With PH1.D / GPT-5.5 / PH1.N

Current repo evidence did not show GPT/OpenAI granting access or making access decisions.

Does current repo use GPT/OpenAI for access decisions? NOT_FOUND.

Does current repo use NLP for role/access candidate extraction? PARTIAL. Blueprints and SimulationExecutor use `IntentDraft` and `FieldKey` values for access actions; PH1.N architecture exists. Direct OpenAI/NLP candidate extraction into Access is not fully proven.

Does current repo use deterministic access only? Gate decision is deterministic.

Future PH1.D/PH1.N proposal path needed:

- GPT-5.5/PH1.D may propose likely role/access intent from messy text.
- PH1.N may extract candidate tenant, user, role, permission, resource, action, risk, and missing fields.
- PH1.X validates route/risk/owner.
- Access/Governance makes deterministic policy decision.

Safety gates:

- OpenAI must not grant access.
- PH1.N candidates must not execute.
- Role names must map to approved templates.
- High-access roles require confirmation and escalation policy.
- Provider-off/fake-provider proof needed before use.

## 16. Interaction With PH1.WRITE

PH1.WRITE ownership for access denial/explanation is underdefined.

Evidence:

- SimulationExecutor returns fail-closed reasons like `ACCESS_SCOPE_VIOLATION` and `ACCESS_AP_REQUIRED`.
- Adapter/public brain surfaces include hardcoded protected fail-closed text such as no simulation/no authority/no protected execution.
- Desktop/iPhone contain fixed explanatory strings for authority/access posture.
- PH1.WRITE master design says final user-facing presentation should be PH1.WRITE-owned.

Current state:

- PH1.WRITE-owned access denial wording: NOT_FOUND.
- Hardcoded access denial/fail-closed wording: FOUND.
- Client-side access posture text: FOUND but render-only.
- Adapter-side protected/access text: FOUND/PARTIAL.

Risks:

- `ACCESS_WRITING_OWNER_RISK`: FOUND/PARTIAL.
- `HARDCODED_ACCESS_DENIAL_RISK`: FOUND.
- `CLIENT_ACCESS_TEXT_RISK`: PARTIAL, mitigated because clients render read-only posture.
- `ADAPTER_ACCESS_TEXT_RISK`: PARTIAL/RISK.

Correct future rule:

PH1.WRITE owns final user-facing wording for access grants, denials, escalations, approvals, protected gate explanations, and safe refusal. Access should emit decisions and reason codes, not become the writing brain.

## 17. Desktop / iPhone / Adapter Boundaries

Desktop behavior:

- `DesktopSessionShellView.swift` renders `authorityStateCard` with cloud-authored authority-state fields.
- Desktop onboarding entry card states it is bounded app-open/invite-open onboarding entry only and does not locally activate invites or alter authority.
- Desktop has `submitDesktopAccessProvisionCommit`, which stages a bounded request through canonical runtime bridge rather than local access policy.
- Status: PARTIAL render/submit-only evidence.
- `DESKTOP_ACCESS_AUTHORITY_RISK`: NOT_FOUND as local access decision; keep monitored because submit affordances exist.

iPhone behavior:

- `SessionShellView.swift` parses explicit entry/invite/open URL metadata including `access_engine_instance_id`.
- It renders onboarding artifact/access summary and says it does not activate access engines or produce runtime requests locally.
- Status: PARTIAL render-only evidence.
- `IPHONE_ACCESS_AUTHORITY_RISK`: NOT_FOUND as local access decision; keep monitored.

Adapter behavior:

- `OnboardingContinueAdapterResponse` exposes `access_engine_instance_id`.
- Adapter has `protected_fail_closed` and payroll/private/protected compatibility classifiers.
- Adapter serves web/app transport surfaces and public brain traces.
- Status: PARTIAL/RISK.
- `ADAPTER_ACCESS_AUTHORITY_RISK`: PARTIAL because Adapter has old semantic classification logic. It appears fail-closed, but PH1.X/Access should own semantics and policy.

Runtime behavior:

- SimulationExecutor and PH1.F storage are the active access decision places.
- App ingress transports access step-up and onboarding access provision requests.

Old compatibility routes:

- Adapter protected/payroll helpers.
- PH1.LINK role proposal/dual role conflict drafts.
- ONB access provision commit.

These must remain until active-caller proof supports retirement.

## 18. Security / Privacy / Consent Model

| Security / Privacy Area | Repo Evidence | Status | Notes |
|---|---|---|---|
| tenant boundaries | tenant id on access records, tests | FOUND | Strong current evidence. |
| workspace boundaries | no first-class access workspace id | NOT_FOUND | SECURITY_GAP. |
| user identity proof | `identity_verified`, FK to identities, verification level | FOUND/PARTIAL | Identity rows and flags exist; identity proof semantics wider than access. |
| role permissions | `role_template_id`, permissions JSON, AP profiles | PARTIAL | Template registry incomplete. |
| field-level permissions | `financial_auth` only | PARTIAL | Salary/payroll/HR/customer/supplier fields need classification. |
| resource-level permissions | requested action string, allow-list | PARTIAL | Resource ids and object scopes incomplete. |
| access expiration | override `expires_at` | PARTIAL | Access instance expiration not found. |
| access revocation | revoke override status, lifecycle suspended | PARTIAL | No full revocation workflow found. |
| access escalation | gate escalation, board policy/vote | PARTIAL | Case lifecycle/threshold resolver missing. |
| break-glass/emergency access | no evidence found | NOT_FOUND | SECURITY_GAP. |
| consent | ONB terms and consent adjacent | PARTIAL | Access consent model not first-class. |
| audit | append-only ledgers, step-up audit | PARTIAL | Generic gate/private read audit missing. |
| idempotency | access instance, overrides, AP rows, votes | FOUND | Tests prove dedupe. |
| least privilege | access mode ranks and allow-list | PARTIAL | Legacy rows without allow-list allow all. |
| sensitive field classification | `financial_auth` flag | PARTIAL | Broader classification missing. |
| salary/payroll/HR/finance protections | Adapter payroll fail-closed + financial flag | PARTIAL | Need canonical Access/PH1.X owner. |
| customer/supplier protections | no explicit model found | NOT_FOUND | SECURITY_GAP. |
| private memory access | PH1.M architectures, not Access extraction | PARTIAL | Needs Access/Memory reconciliation. |
| external send permissions | SimulationExecutor `DELIVERY_SEND` gate for link delivery, BCAST docs | PARTIAL | Full recipient/channel consent is delivery gap. |

## 19. Access State Machine

RECONSTRUCTED_FROM_REPO_EVIDENCE:

Access instance states:

- `Restricted`: instance exists but identity or restrictions require step-up; gate returns `ESCALATE`.
- `Active`: instance can allow/deny/escalate based on requested action, mode, sensitive flag, and device trust.
- `Suspended`: gate returns `DENY`.

Access gate decision states:

- `Allow`: requested action present or legacy allow, requested mode within effective mode, no sensitive/device block.
- `Deny`: missing instance, user scope mismatch, suspended instance, action not allowed, sensitive fields blocked.
- `Escalate`: restricted/unverified, device untrusted, mode upgrade required.

AP schema states:

- `Draft`: created/updated but not active.
- `Active`: current projection points to version.
- `Retired`: version retired; current projection removed if retiring active version.

AP authoring review states:

- `NeedsChannelChoice`: contract-defined; storage channel commit moves to review in progress.
- `ReviewInProgress`: review channel chosen.
- `PendingActivationConfirmation`: confirmation pending after rule actions.
- `ConfirmedForActivation`: activation can capture review lineage.
- `Declined`: further rule action append blocked.

Override states:

- `Active`: one-shot/temporary/permanent currently effective if window valid.
- `Expired`: written expired if expiration is before now.
- `Revoked`: revoke override row.

Board policy states:

- `Draft`, `Active`, `Retired`.

Board vote states:

- Vote row is append-only with value `Approve` or `Reject`.
- Threshold state is blueprint-level (`PENDING | SATISFIED | REJECTED`) but not found as a runtime table.

Possible requested states not proven:

- `Requested`: PARTIAL through blueprints/IntentDraft.
- `PendingApproval`: PARTIAL through `Escalate` and board vote rows.
- `Granted`: equivalent to active access instance/allow decision, but no explicit grant state.
- `Expired`: partial through overrides only.
- `Revoked`: partial through override rows only.
- `Conflict`: NOT_FOUND as access state, though link dual role conflict draft exists.
- `NeedsStepUp`: equivalent to escalation trigger.
- `NeedsSimulation`: PH1.X/SimulationExecutor guard, not Access state.
- `NeedsAuthority`: Authority state, not Access state.
- `NeedsConfirmation`: AP authoring confirmation states and blueprints.

## 20. Error Handling And Reason Codes

| Required Error / Reason | Repo Truth | Status | Notes |
|---|---|---|---|
| access denied | `AccessDecision::Deny`, `ACCESS_SCOPE_VIOLATION` | FOUND | Used by gate and SimulationExecutor. |
| tenant mismatch | override tenant validation; schema-chain tenant read tests | FOUND | Contract violation or FK fail. |
| workspace mismatch | no workspace scope found | NOT_FOUND | DESIGN_GAP. |
| role not found | role id bounded; no registry check except position compile | PARTIAL | Role template registry missing. |
| permission missing | `ACTION_NOT_ALLOWED` | FOUND | Allow-list denies action. |
| authority missing | protected fail-closed reports/adapter strings | PARTIAL | Authority is broader than Access. |
| simulation missing | SimulationExecutor simulation catalog guard | FOUND/PARTIAL | Access extraction sees guard, not full simulation doc. |
| confirmation missing | AP activation confirmed review requirement | PARTIAL | General confirmation missing. |
| access expired | override expires/status | PARTIAL | Instance expiration missing. |
| access revoked | revoke override | PARTIAL | Direct revoke workflow missing. |
| role conflict | PH1.LINK dual role conflict draft | PARTIAL | Not Access-owned state. |
| dual role conflict | PH1.LINK dual role conflict draft | PARTIAL | Escalation draft only. |
| user not found | identity FK violation | FOUND | Upsert requires identity except ONB creates identity row. |
| identity confidence low | restricted/unverified escalation | PARTIAL | Confidence scoring not found. |
| voice/device evidence insufficient | device untrusted, step-up defer/refuse | PARTIAL | Voice-specific evidence not direct. |
| private data denied | `SENSITIVE_FIELDS_BLOCKED` | FOUND/PARTIAL | Only financial flag. |
| protected execution denied | SimulationExecutor fail closed | PARTIAL | Broader authority path. |
| escalation required | `ACCESS_AP_REQUIRED`, `StepUpProofRequired` | FOUND | Gate escalation. |
| approval required | `AccessEscalationTrigger::ApApprovalRequired` | FOUND/PARTIAL | Approval case lifecycle partial. |
| unsupported access type | unknown | NOT_FOUND | No normalized type matrix. |
| client route mismatch | app ingress route validation | PARTIAL | Not access-specific. |

## 21. Audit / Provenance / Evidence

| Audit Question | Repo Truth | Status |
|---|---|---|
| Is access request audited? | Step-up START/FINISH audit via `ph1access_capreq_step_up_audit_commit`; generic access request audit not found. | PARTIAL/AUDIT_GAP |
| Is access grant audited? | `access_instances` row and AP ledgers; PH1.J grant event not found. | PARTIAL |
| Is access denial audited? | Gate returns decision; SimulationExecutor fail closed; generic PH1.J denial audit not found. | AUDIT_GAP |
| Is role/template assignment audited? | Access instance rows and compile lineage persist role/template refs. | PARTIAL |
| Is revocation audited? | Revoke override row persists; explicit audit event not found. | PARTIAL |
| Is protected execution access check audited? | Step-up audit found; generic simulation access check audit partial. | PARTIAL |
| Are private data reads audited? | Not found. | AUDIT_GAP |
| Are onboarding access handoffs audited? | ONB session status and access instance refs persist; explicit PH1.J handoff audit not proven. | PARTIAL |
| Are link/access proposals audited? | PH1.LINK role-proposal/dual-role drafts have storage refs; access audit link not proven. | PARTIAL |
| Are voice/device evidence refs recorded? | Access instance stores verification/device trust levels; ONB stores voice/persona/device prerequisites. | PARTIAL |
| Are tenant/workspace/user refs recorded? | Tenant/user refs recorded; workspace not found. | FOUND/PARTIAL |
| Are client/adapter access events audited? | Clients render; Adapter traces internal history; explicit access audit not found. | AUDIT_GAP |

## 22. Current Tests / Smokes / Acceptance

| Test / Smoke | Path | What It Proves | What It Does Not Prove | Status |
|---|---|---|---|---|
| `at_access_db_01_tenant_isolation_enforced` | `crates/selene_storage/tests/ph1_access_ph2_access/db_wiring.rs` | Tenant/user isolation and override tenant match. | Workspace/resource scope. | FOUND |
| `at_access_db_02_append_only_enforced` | same | Access overrides are append-only. | All access ledgers immutable under SQL. | FOUND |
| `at_access_db_03_idempotency_dedupe_works` | same | Instance/override idempotency dedupe. | Live DB concurrency. | FOUND |
| `at_access_db_04_current_table_no_ledger_rebuild_required` | same | Gate reads current table without rebuilding ledger. | Full audit of decision. | FOUND |
| `at_access_db_05_gate_missing_instance_fails_closed` | same | Missing access instance denies. | User-facing explanation. | FOUND |
| `at_access_db_06_gate_deny_when_requested_action_not_allowed` | same | Allow-list denies disallowed action. | Field/resource permissions. | FOUND |
| `at_access_db_07_gate_escalate_on_mode_upgrade_required` | same | Mode upgrade returns escalation. | End-to-end approval resolution. | FOUND |
| `at_access_db_08_gate_user_scope_mismatch_fails_closed` | same | User mismatch denies. | Tenant/workspace hierarchy. | FOUND |
| `at_access_db_09_gate_allow_path_is_deterministic_across_retries` | same | Gate allow is deterministic. | SQL persistence/live DB. | FOUND |
| `at_access_db_10_deny_by_default_when_schema_chain_missing_global_ap` | same | Missing global AP chain fails closed. | User recovery. | FOUND |
| `at_access_db_11_ap_version_pin_and_replay_determinism` | same | Compile lineage/version pin and replay determinism. | Semantic template correctness. | FOUND |
| `at_access_db_12_overlay_merge_deterministic` | same | Overlay set ordering deterministic. | Overlay semantics beyond refs. | FOUND |
| `at_access_db_13_position_binding_required_for_compile` | same | Position binding FK required when compile references position. | Position policy completeness. | FOUND |
| `at_access_db_14_escalation_n_of_m_and_board_quorum_vote_paths` | same | Board policy current and votes persist idempotently. | Threshold resolver / AP approval execution. | FOUND |
| `at_access_db_15_override_lifecycle_types_persist` | same | One-shot/temp/permanent/revoke overrides persist. | Override resolution beyond grant_mode. | FOUND |
| `at_access_db_16_tenant_isolation_enforced_for_schema_chain_reads` | same | Tenant scoped schema-chain reads. | Workspace scope. | FOUND |
| `at_access_db_17_ap_authoring_review_channel_persists_and_dedupes` | same | Review channel current/ledger and dedupe. | Live review UI. | FOUND |
| `at_access_db_18_ap_authoring_rule_action_requires_review_state_and_dedupes` | same | Rule action requires review state and dedupes. | Rule semantic validation. | FOUND |
| `at_access_db_19_ap_authoring_confirm_requires_rule_actions_and_blocks_after_decline` | same | Confirmation requires rule actions; declined blocks later rule action. | Human explanation. | FOUND |
| `at_access_db_20_ap_activation_captures_authoring_lineage_when_confirmed` | same | Activation captures review event/rule action lineage. | Live AP activation acceptance. | FOUND |
| `at_access_db_21_ap_activation_fails_closed_without_confirmed_review_state` | same | AP activation fails without confirmed review. | Full approval policy. | FOUND |
| `at_sim_exec_19_access_schema_manage_gate_allow_returns_gate_passed` | `crates/selene_os/src/simulation_executor.rs` | Access schema manage gate can pass. | Commit write itself. | FOUND |
| `at_sim_exec_20_access_escalation_vote_access_deny_blocks_governed_commit` | same | Suspended/denied access blocks governed commit. | Full board case lifecycle. | FOUND |
| `at_sim_exec_21_access_instance_compile_access_escalate_requires_approval_before_commit` | same | Escalation blocks compile without approval. | Approval resolution. | FOUND |
| `at_sim_exec_22_access_schema_manage_missing_review_channel_fails_closed` | same | Missing access review channel fails closed. | PH1.WRITE clarification. | FOUND |
| `at_sim_exec_23_access_schema_manage_read_out_loud_gate_allow_returns_gate_passed` | same | Read-out-loud review channel accepted. | TTS/read-aloud UI. | FOUND |
| `at_sim_exec_24_access_schema_manage_activate_missing_rule_action_fails_closed` | same | Activate path requires rule action fields. | Full AP activation commit. | FOUND |
| `at_sim_exec_16*` access step-up tests | same | Step-up can continue/defer/refuse and audit. | Real biometric/passcode integration. | FOUND |
| Adapter payroll/protected tests in reports | `docs/reports/STAGE_8_FRESH_MEMORY_REAL_VOICE_PROOF.md` | Protected payroll fails closed and public payroll remains public. | Canonical Access field gate. | PARTIAL |

## 23. Old Paths / Compatibility / Wrong-Owner Risks

| Path / Symbol | Current Status | Correct Canonical Owner | Retirement Condition | Active-Caller Check Needed |
|---|---|---|---|---|
| `ph1onb_access_instance_create_commit` direct access upsert | ACTIVE/PARTIAL | PH1.ONB coordinates; Access/Governance owns policy/template | Access handoff proof with template validation and no ONB policy guessing | Yes |
| `role_to_default_access_mode` string inference | ACTIVE/RISK | Access role/template registry | Registry-backed role template mode proof | Yes |
| `role_template_is_owner_or_admin` string helper | ACTIVE/RISK | Access/Governance role hierarchy | Owner/admin permission proof without string contains | Yes |
| `baseline_permissions_json` legacy allow-all behavior when no allow-list | ACTIVE/RISK | Access permission matrix | Explicit migration of legacy rows or compatibility ledger | Yes |
| Adapter `payroll_governed_business_intent` / `payroll_private_business_read_intent` | ACTIVE COMPATIBILITY/RISK | PH1.X + Access + domain data owners | Canonical private/protected route proof and equivalent tests | Yes |
| Hardcoded protected fail-closed user text | ACTIVE/RISK | PH1.WRITE | PH1.WRITE access/protected denial boundary tests | Yes |
| PH1.LINK role proposal / dual role conflict drafts | ACTIVE/PARTIAL | PH1.LINK proposes; Access/Governance decides | Prove role proposals cannot grant access and route to Access approval | Yes |
| Desktop `submitDesktopAccessProvisionCommit` | ACTIVE/PARTIAL | Desktop transport/render only; OS/Access decide | Render/submit-only proof for all access provision paths | Yes |
| iPhone explicit entry access identifier rendering | ACTIVE/PARTIAL | iPhone render only | Render-only proof remains green | Yes |
| Access board vote without threshold resolver | ACTIVE/PARTIAL | Access/Governance approval workflow | Deterministic escalation case resolution proof | Yes |
| Access override writes with caller-provided simulation id | ACTIVE/PARTIAL | SimulationExecutor + Access | Enforce active simulation/approval before every override write | Yes |
| Access schemas simulations DRAFT in catalog | ACTIVE/PARTIAL | Simulation catalog + Access | Active simulation proof before production execution | Yes |
| Stale docs/fix-plan packets | RETAINED_DOC_HISTORY | Grand Architecture Reconciliation | Mark superseded only after reconciliation | Yes |

## 24. Full Functionality Master List

| Functionality | Description | Evidence | Owner | Status | Future Action |
|---|---|---|---|---|---|
| create access instance | Upsert tenant/user access instance | `ph2access_upsert_instance_commit` | PH2.ACCESS | FOUND | Add template validation and audit. |
| check access | Deterministic gate decision | `ph1access_gate_decide` | PH1.ACCESS | FOUND | Wire universal private/protected paths. |
| deny access | Return denial with flags/reason | gate deny branches | PH1.ACCESS | FOUND | PH1.WRITE denial wording. |
| escalate access | Step-up/AP required decisions | gate escalation branches | PH1.ACCESS | FOUND/PARTIAL | Full escalation workflow. |
| update access | Upsert/compile/overrides | storage methods | PH2.ACCESS | PARTIAL | Controlled policy workflow. |
| revoke access | Revoke override / suspend state via upsert | `AccessOverrideType::Revoke`, `AccessLifecycleState::Suspended` | Access/Governance | PARTIAL | Explicit revocation state machine. |
| role template mapping | String role template id + default mode helper | `role_template_id`, `role_to_default_access_mode` | Access/Governance | PARTIAL/RISK | Canonical role registry. |
| user role assignment | Access instance stores role template id | `AccessInstanceRecord` | Access/Governance | PARTIAL | Role conflict checks. |
| tenant scope validation | Tenant id and tests | migrations/tests | Access | FOUND | Keep. |
| workspace scope validation | No first-class model found | none | Access/Workspace | NOT_FOUND | Add workspace scope model. |
| onboarding access handoff | ONB creates access instance after gates | PH1.ONB storage/runtime | PH1.ONB + Access | FOUND/PARTIAL | Move policy to Access/Governance. |
| link role proposal | Link drafts role proposal/conflict escalation | PH1.LINK | PH1.LINK + Access | PARTIAL | Route proposal to Access. |
| protected action precheck | SimulationExecutor access gates | OS runtime | PH1.X + Access + Simulation | FOUND/PARTIAL | Full authority proof. |
| private data read check | Sensitive flag + financial auth | `ph1access_gate_decide` | Access | PARTIAL | Field/resource classifier. |
| AP schema draft | Create/update AP schema ledger | `ph1access_ap_schema_lifecycle_commit` | PH1.ACCESS | FOUND | Human review UX. |
| AP schema activate/retire | Current projection update/remove | same | PH1.ACCESS | FOUND | Active simulation proof. |
| AP authoring review channel | Store phone/desktop or read-out-loud review channel | `ph1access_ap_authoring_review_channel_commit` | PH1.ACCESS | FOUND | UX and PH1.WRITE proof. |
| AP rule action | Store rule action rows | `ph1access_ap_authoring_rule_action_commit` | PH1.ACCESS | FOUND | Rule semantic validation. |
| AP confirmation | Store confirmed/pending/declined state | `ph1access_ap_authoring_confirm_commit` | PH1.ACCESS | FOUND | JD live acceptance. |
| tenant overlay | Store active overlay versions | `ph1access_ap_overlay_update_commit` | PH1.ACCESS | FOUND | Overlay semantics. |
| board policy | Store active board policy versions | `ph1access_board_policy_update_commit` | PH1.ACCESS | FOUND | Threshold resolver. |
| board vote | Append board votes | `ph1access_board_vote_commit` | PH1.ACCESS | FOUND | Resolve case. |
| access override | Append one-shot/temp/perm/revoke override | `ph2access_apply_override_commit` | PH2.ACCESS | FOUND/PARTIAL | Enforce approval/ref lifecycle. |
| compile lineage | Persist AP/overlay/position lineage to instance | `ph1access_instance_compile_commit` | PH1.ACCESS/PH2.ACCESS | FOUND | Role/resource policy expansion. |
| step-up audit | Audit access step-up start/finish | `ph1access_capreq_step_up_audit_commit` | Access + PH1.J | FOUND/PARTIAL | Broader gate audit. |
| client rendering | Show access/authority identifiers | Desktop/iPhone Swift | Clients | PARTIAL | Render-only proof. |
| adapter transport | Expose access ids/fail-closed traces | Adapter | Adapter | PARTIAL/RISK | Remove wrong-owner classification later. |

## 25. Comparison To Master Architecture

Identity + Access + Authority Spine:

- Current repo supports the spine’s law that Voice ID is evidence, Access is permission/scope, and Authority/Simulation remain separate.
- Access gate is deterministic and fail-closed in many cases.
- Missing: full authority packet reconciliation and broad field/resource policy.

Global Request Decision Lattice:

- Current access can consume resolved requested actions.
- Lattice must own ambiguity/risk/lane resolution before access.
- Adapter compatibility phrase logic should migrate to PH1.X/PH1.N.

PH1.D Proposal Gateway:

- No OpenAI/GPT access decision path was found.
- Future PH1.D can propose role/permission intent but must not decide.

PH1.N Meaning Unravelling:

- Current access intent fields exist through `IntentDraft`.
- Full messy role/access candidate extraction is future work.

PH1.WRITE Human Presentation:

- Current denial/status wording is underdefined and partly hardcoded.
- PH1.WRITE should own final access-denial/explanation language.

PH1.ONB Onboarding Journey:

- ONB currently creates access instances after deterministic prerequisites.
- Future handoff must preserve ONB progression while moving policy/template decisions to Access/Governance.

PH1.LINK Link Journey:

- Link role proposal/dual-role conflict drafts exist, but link does not grant access.
- Future link journey must call Access/Governance for role templates and permission gates.

PH1.VOICE.ID Human Presence:

- Voice/device evidence is posture only.
- Access step-up uses verification/device trust but no voice authority found.

PH1.REM Reminder Journey:

- Reminder mutations are access-gated in SimulationExecutor.
- REM does not own access/timing authority beyond its own domain.

PH1.BCAST / PH1.DELIVERY:

- Link delivery and BCAST policy updates are access-gated in SimulationExecutor.
- Delivery/BCAST do not grant access.

Master Access Template / Role / Permission stack:

- AP schemas, overlays, board policies, compile lineage are real foundations.
- Semantic role/template registry is incomplete.

Tenant / Workspace Governance:

- Tenant scope is implemented.
- Workspace scope is missing.

Desktop/iPhone render-only boundary:

- Clients render access/authority state and bounded onboarding entry/access id context.
- No local access decision found.

Adapter transport-only boundary:

- Adapter transports and traces but contains retained semantic classification for protected/payroll.
- Future owner cleanup required.

Old Compatibility Path Retirement:

- Do not delete old paths until canonical PH1.X/Access/PH1.WRITE replacements and active-caller proof exist.

## 26. Gaps / Missing Pieces

| Gap | Evidence | Risk | Recommended Future Action | Priority |
|---|---|---|---|---|
| missing master access template semantics | AP schemas store JSON payloads; semantics partial | Template names may not enforce policy | Master Access Template Registry | High |
| missing role template registry | role strings and string helpers | Role text could imply permissions | Role / Permission Matrix | High |
| missing permission matrix | allow-list JSON and financial flag only | Incomplete least privilege | Permission matrix + resource scopes | High |
| missing per-user access live SQL proof | migrations/tests exist, no live DB acceptance | Production persistence uncertainty | SQL persistence/JD live proof | Medium |
| missing workspace scope | no workspace id on access records | Cross-workspace leak | Tenant / Workspace Scope Integration | High |
| missing company/entity scope | tenant company exists but Access partial | Company admin overreach | Company/entity access model | High |
| missing field-level permission model | only `financial_auth` | Salary/HR/customer/supplier data leak | Field-level permission model | High |
| missing private data read gate | sensitive boolean not universally wired | Private reads may bypass Access | Private Data Read Gate | High |
| missing protected execution full authority gate | access checks selected paths only | Access mistaken for execution authority | Protected Execution Gate reconciliation | High |
| missing access denial PH1.WRITE boundary | hardcoded text | Poor/unsafe explanations | PH1.WRITE Access Denial Boundary | Medium |
| missing PH1.D/PH1.N access proposal path | no provider path found | Messy roles may be misread | Proposal shell with deterministic validation | Medium |
| missing access audit | ledgers but generic gate/private read audit gaps | Insufficient proof | Access Audit Evidence Pack | High |
| missing revocation/expiration | overrides only; instance expiration absent | Stale access | Revocation/Expiration/Suspension Proof | High |
| missing approval/escalation workflow | votes persist, threshold resolver partial | Escalation cannot complete safely | Approval / Escalation Workflow | High |
| missing break-glass/emergency policy | not found | Unsafe emergency access | Break-glass design with authority | Medium |
| missing Desktop/iPhone render-only proof | read-only copy exists, full proof partial | Client authority drift | Render-only access proof | Medium |
| missing Adapter transport-only proof | Adapter classification helpers exist | Wrong-owner access decisions | Adapter retirement ledger | High |
| missing JD live acceptance | no live acceptance found | Unproven product readiness | JD Live Access Acceptance Pack | High |

## 27. Recommended Future Build Slices

Based on repo truth, recommended future build slices:

1. Master Access Repo-Truth Activation Pack
2. Access Contract / State Machine Normalization
3. Master Access Template Registry
4. Role / Permission Matrix
5. Per-User Access Instance Proof
6. Tenant / Workspace Scope Integration
7. Onboarding Access Handoff Proof
8. Link Role Proposal / Access Context Proof
9. Voice/Device Evidence-To-Access Boundary Proof
10. Private Data Read Gate
11. Protected Execution Gate
12. PH1.WRITE Access Denial / Explanation Boundary
13. PH1.D + PH1.N Access Candidate Proposal Shell
14. Approval / Escalation Workflow
15. Access Audit Evidence Pack
16. Revocation / Expiration / Suspension Proof
17. Desktop/iPhone Render-Only Access Proof
18. Adapter Transport-Only Access Proof
19. JD Live Access Acceptance Pack

Additional repo-truth-driven slices that should be considered:

20. Access AP Authoring Review UX + Confirmation Proof
21. Board Vote Threshold Resolver Proof
22. Access Override Approval Ref Enforcement
23. Legacy Allow-List Compatibility Retirement Ledger
24. Payroll/Salary/HR Sensitive Field Classification Proof
25. Workspace/Resource ID Access Scope Migration Plan

## 28. What Codex Must Not Do

- do not invent access behavior
- do not create duplicate access engine
- do not let onboarding grant access policy
- do not let link activation grant access
- do not let Voice ID grant authority
- do not let GPT-5.5/OpenAI grant access
- do not let Desktop/iPhone decide access
- do not let Adapter decide access
- do not treat role names as permissions without template proof
- do not expose salary/payroll/HR/customer/supplier/private data without access gate
- do not execute protected actions from access check alone
- do not bypass Authority + Simulation for protected execution
- do not bypass PH1.WRITE for user-facing denial/explanation where unsafe
- do not delete old paths before proof
- do not implement from this extraction document alone

## 29. Final Extracted Architecture Sentence

Selene Master Access Engine is the governed scope and permission boundary: it may decide whether a user, identity, tenant, workspace, role, or resource scope allows a read, setup, or action path, but it must not replace PH1.X routing, PH1.D proposals, PH1.WRITE presentation, PH1.LINK activation, PH1.ONB onboarding, PH1.VOICE.ID evidence, Authority validation, SimulationExecutor execution, Desktop rendering, or Adapter transport.
