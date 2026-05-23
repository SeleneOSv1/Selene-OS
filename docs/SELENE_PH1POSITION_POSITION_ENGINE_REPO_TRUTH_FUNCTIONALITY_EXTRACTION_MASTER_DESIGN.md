# Selene PH1.POSITION Position Engine — Repo-Truth Functionality Extraction Master Design

REPO_TRUTH_EXTRACTION
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

## 0. Authority and Scope

AGENTS.md controls execution.

This is docs-only repo-truth extraction.

No runtime code was changed.

This document does not authorize implementation.

The document reconstructs current Position Engine / Position Requirements / Job Role / Position-to-Access / Position-to-Onboarding design and functionality from repo evidence.

Future implementation/refactor/retirement requires explicit build instruction, approved file scope, tests, backend evidence, JD approval, and where visible JD live proof.

This extraction does not claim that current PH1.POSITION is the final future architecture. It separates current repo truth from missing, partial, and future design needs. Missing areas are marked NOT_FOUND, PARTIAL, UNKNOWN, REPO_TRUTH_NEEDED, DESIGN_GAP, TEST_GAP, OWNER_GAP, AUDIT_GAP, or SECURITY_GAP.

## 1. Executive Summary

PH1.POSITION is present in current repo truth as an authoritative position lifecycle and position requirements schema owner. Its strongest evidence is `crates/selene_kernel_contracts/src/ph1position.rs`, `crates/selene_os/src/ph1position.rs`, `crates/selene_storage/src/ph1f.rs`, `crates/selene_storage/src/repo.rs`, `docs/DB_WIRING/PH1_POSITION.md`, `docs/ECM/PH1_POSITION.md`, `docs/BLUEPRINTS/POSITION_MANAGE.md`, `docs/BLUEPRINTS/ONB_SCHEMA_MANAGE.md`, `docs/08_SIMULATION_CATALOG.md`, and `crates/selene_storage/tests/ph1_position/db_wiring.rs`.

PH1.POSITION appears to be a real current repo owner, not only a design placeholder. It supports deterministic tenant/company-scoped position draft creation, authority/company validation, compensation-band policy check, activation, suspension, retirement, append-only lifecycle events, versioned position requirements schema create/update/activate, and onboarding requirement backfill handoff signaling through `PositionSchemaApplyScope::CurrentAndNew`.

There is no standalone `crates/selene_engines/src/ph1position.rs` file. Current execution is OS/runtime + PH1.F storage backed, with a typed kernel contract and storage repo trait. The authoritative engine inventory still lists PH1.POSITION as authoritative governance/storage.

PH1.POSITION defines position/job truth only in a bounded current form: title, department, jurisdiction, schedule type, tenant, company, permission profile reference, compensation band reference, and lifecycle state. It does not currently prove a full enterprise position hierarchy, department hierarchy, manager/reporting-line graph, job-family registry, workspace scope, payroll group truth, roster group truth, or promotion/demotion engine.

PH1.POSITION defines onboarding requirements through versioned field specs. Employee onboarding can read active position requirement schemas, pin schema refs on session start, compute required fields, and derive photo/sender-verification gates from `DocRequired` fields. ONB is read-only against position schema truth and starts backfill campaigns when a schema is activated for current and new sessions.

PH1.POSITION does not define access permissions directly. Current position rows contain `permission_profile_ref`, and Access compile lineage can carry and validate an optional `position_id`. This is PARTIAL position-to-access evidence, not proof that Position grants access. Correct future rule remains: Position defines the job; Access grants the permissions.

PH1.POSITION has a compensation-band reference and a band policy check, but Payroll/HR truth is not present as a Position-owned implementation. It has `PositionScheduleType`, but Scheduler/Roster truth is not present as a Position-owned implementation. Current repo truth supports references and policy checks, not ownership of payroll, salary, schedule, roster, workload, leave, termination, or resignation truth.

Active:

- PH1.POSITION kernel contract.
- OS runtime execution.
- PH1.F storage functions.
- typed repo trait.
- tenant company upsert/read under PH1.POSITION ECM.
- position lifecycle current + append-only event records in PH1.F.
- position requirements schema ledger/current in PH1.F and SQL migration.
- ONB read-only consumption of active position requirement schema.
- ONB backfill handoff when `CurrentAndNew` is selected.
- storage/runtime/simulation tests.

Partial:

- SQL persistence for position requirements schema and ONB backfill tables exists, but direct SQL `CREATE TABLE` evidence for `positions` and `position_lifecycle_events` was not found in migrations.
- Position-to-Access exists as `permission_profile_ref` plus `AccessCompiledLineageRef.position_id` validation, but not as a complete access-template co-authoring or recompile flow.
- Position-to-Onboarding exists for employee required fields/gates, but not as a full sender-prefill/receiver-field taxonomy.
- company/entity scope exists through `tenant_companies`, but workspace scope is not present in PositionRecord.
- industry/country/company-size overlays exist as selector snapshot fields, not as a full overlay governance engine.

Unclear or missing:

- standalone Position engine crate.
- full position hierarchy and reporting line.
- department hierarchy and job-family registry.
- dynamic one-time field override.
- salary/payroll/HR owner integration beyond compensation-band reference.
- Scheduler/Roster owner integration beyond schedule type.
- Desktop/iPhone position UI.
- Adapter position routes.
- PH1.WRITE-owned position guidance.
- PH1.D/GPT-5.5 current position proposal path.
- JD live acceptance proof for position journeys.

The biggest risks/gaps are role/position/access conflation, treating `permission_profile_ref` or role strings as access authority, missing workspace/company hierarchy, incomplete payroll/roster owner boundaries, no full position hierarchy, no PH1.WRITE wording boundary, and no live product UI for position creation/update/retirement.

## 2. Repo Evidence Inventory

| Evidence Area | File / Path | Symbols / Routes / Tables | Status | Notes |
|---|---|---|---|---|
| Kernel contract | `crates/selene_kernel_contracts/src/ph1position.rs` | `PH1POSITION_CONTRACT_VERSION`, `PositionRecord`, `PositionRequest`, `Ph1PositionRequest`, `PositionRequirementFieldSpec`, `PositionSchemaSelectorSnapshot` | FOUND | Canonical contract for position lifecycle and requirements schema. |
| Simulation IDs | `crates/selene_kernel_contracts/src/ph1position.rs` | `POSITION_SIM_001_CREATE_DRAFT`, `POSITION_SIM_002_VALIDATE_AUTH_COMPANY`, `POSITION_SIM_003_BAND_POLICY_CHECK`, `POSITION_SIM_004_ACTIVATE_COMMIT`, `POSITION_SIM_005_RETIRE_OR_SUSPEND_COMMIT`, `POSITION_REQUIREMENTS_SCHEMA_CREATE_DRAFT`, `POSITION_REQUIREMENTS_SCHEMA_UPDATE_COMMIT`, `POSITION_REQUIREMENTS_SCHEMA_ACTIVATE_COMMIT` | FOUND | Request variant validates matching simulation id/type. |
| Lifecycle enums | same | `PositionLifecycleState::{Draft, Active, Suspended, Retired}`, `PositionLifecycleAction`, `PositionRequestedAction` | FOUND | Current state machine is bounded. |
| Schedule enum | same | `PositionScheduleType::{FullTime, PartTime, Contract, Shift}` | FOUND | Position-level schedule type only; not Scheduler/Roster truth. |
| Requirement field model | same | `PositionRequirementFieldType`, `PositionRequirementRuleType`, `PositionRequirementSensitivity`, `PositionRequirementExposureRule`, `PositionRequirementEvidenceMode` | FOUND | Typed field definitions for onboarding requirements. |
| Apply scope | same | `PositionSchemaApplyScope::{NewHiresOnly, CurrentAndNew}`, `requires_backfill_handoff()` | FOUND | Current-and-new schema activation signals backfill handoff. |
| OS runtime | `crates/selene_os/src/ph1position.rs` | `Ph1PositionRuntime::run`, `reason_codes::*`, PH1.J `audit_transition` | FOUND | Executes contract variants against PH1.F and emits PH1.J state-transition audits. |
| Storage current | `crates/selene_storage/src/ph1f.rs` | `positions`, `position_lifecycle_events`, `position_requirements_schema_ledger`, `position_requirements_schema_current` | FOUND | In-memory PH1.F current and ledger structures. |
| Storage functions | `crates/selene_storage/src/ph1f.rs` | `ph1position_create_draft`, `ph1position_validate_auth_company_draft`, `ph1position_band_policy_check_draft`, `ph1position_activate_commit`, `ph1position_retire_or_suspend_commit` | FOUND | Deterministic position lifecycle operations. |
| Schema functions | `crates/selene_storage/src/ph1f.rs` | `ph1position_requirements_schema_create_draft`, `ph1position_requirements_schema_update_commit`, `ph1position_requirements_schema_activate_commit` | FOUND | Versioned requirement schema lifecycle. |
| Repo trait | `crates/selene_storage/src/repo.rs` | `Ph1PositionRepo`, `ph1position_*_row`, `ph1tenant_company_*_row` | FOUND | Typed row API over PH1.F. |
| SQL migration | `crates/selene_storage/migrations/0014_position_requirements_schema_and_backfill_tables.sql` | `position_requirements_schema_ledger`, `position_requirements_schema_current`, `onboarding_requirement_backfill_campaigns`, `onboarding_requirement_backfill_targets` | PARTIAL | Requirements schema/backfill tables exist. Direct SQL tables for `positions` and `position_lifecycle_events` were not found in migrations. |
| DB wiring | `docs/DB_WIRING/PH1_POSITION.md` | position current/ledger ownership, schema current/ledger ownership, acceptance tests | FOUND | Declares PH1.POSITION DB ownership and invariants. |
| ECM | `docs/ECM/PH1_POSITION.md` | `PH1POSITION_CREATE_DRAFT_ROW`, `PH1POSITION_REQUIREMENTS_SCHEMA_*`, `PH1POSITION_APPEND_ONLY_GUARD` | FOUND | Capability ownership and failure modes. |
| Simulation catalog | `docs/08_SIMULATION_CATALOG.md` | eight PH1.POSITION simulation records | FOUND | DRAFT/COMMIT semantics, roles, confirmations, side effects. |
| Blueprint | `docs/BLUEPRINTS/POSITION_MANAGE.md` | `POSITION_MANAGE`, PH1.C -> PH1.NLP -> PH1.X -> Access -> PH1.POSITION | FOUND | Orchestration-only lifecycle journey. |
| Schema blueprint | `docs/BLUEPRINTS/ONB_SCHEMA_MANAGE.md` | `ONB_SCHEMA_MANAGE`, PH1.POSITION schema steps, PH1.ONB backfill step | FOUND | Establishes Position as schema truth, ONB as executor/backfill owner. |
| ONB integration | `crates/selene_storage/src/ph1f.rs` | `ph1link_schema_required_fields`, `ph1onb_validate_employee_position_prereq`, `ph1onb_required_verification_gates_for_token` | FOUND | Employee onboarding consumes active position schema and validates active position/company. |
| ONB live sequence | `crates/selene_os/src/ph1onb.rs` | `OnbPositionLiveRequest`, `OnbPositionLiveResult`, `run_position_live_sequence` | FOUND | ONB runtime can run create/validate/policy/activate sequence through PH1.POSITION. |
| Simulation executor | `crates/selene_os/src/simulation_executor.rs` | `SimulationExecutionOutput::Position`, `execute_position`, `execute_onb_position_live_sequence` | FOUND | Simulation executor can dispatch PH1.POSITION. |
| Access relation | `crates/selene_kernel_contracts/src/ph1access.rs`, `crates/selene_storage/src/ph1f.rs`, `docs/BLUEPRINTS/ACCESS_INSTANCE_COMPILE_REFRESH.md` | `AccessCompiledLineageRef.position_id`, `ph1access_instance_compile_commit` validates `positions` contains position id | PARTIAL | Access compile can bind to position id. Not proof of complete Position-to-Access template co-authoring. |
| Adapter | `crates/selene_adapter/src/lib.rs`, `crates/selene_adapter/src/bin/http_adapter.rs` | `seed_employee_company_and_position`, seeded `PositionRecord`, onboarding tests with `position_id` | PARTIAL | Adapter tests seed position data for invite/onboarding. No direct `/position` route found. |
| Desktop | `apple/mac_desktop` | direct position UI/search | NOT_FOUND | No direct position surface found. |
| iPhone | `apple/iphone` | direct position UI/search | NOT_FOUND | No direct position surface found. |
| Historical packets | `docs/16_PH1_POSITION_STRICT_FIX_PLAN_PACKET.md`, `docs/23_PH1_POSITION_SCHEMA_OWNERSHIP_STRICT_FIX_PLAN_PACKET.md`, `docs/26_PH1_POSITION_SCHEMA_OWNERSHIP_STRICT_FIX_PLAN_PACKET.md` | strict fix plan evidence | FOUND | Historical build records show contract/storage/runtime parity and test closure. |
| Build ledger | `docs/03_BUILD_LEDGER.md` | PH1_POSITION_PACKET steps | FOUND | Records prior PASS gates for kernel contract, repo parity, storage parity, runtime parity, migration parity, and tests. |
| DB ownership matrix | `docs/10_DB_OWNERSHIP_MATRIX.md` | PH1.POSITION row | FOUND | Confirms position lifecycle + requirements schema ownership. |
| Authoritative inventory | `docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md` | PH1.POSITION row | FOUND | Lists PH1.POSITION as authoritative governance/storage. |

## 3. Current Position Owner Map

| Responsibility | Current Owner / File | Correct Future Owner Hypothesis | Status | Notes |
|---|---|---|---|---|
| position creation | PH1.POSITION: `PositionCreateDraftRequest`, `ph1position_create_draft` | PH1.POSITION | FOUND | Creates Draft row; requires active tenant company and actor identity. |
| position update | PH1.POSITION schema update only; no general position field update request found | PH1.POSITION with approval/gate | PARTIAL | Requirements schema update exists; title/department/jurisdiction update does not appear as current capability. |
| position activation | PH1.POSITION: `PositionActivateCommitRequest` | PH1.POSITION | FOUND | Draft/Suspended -> Active. |
| position retirement | PH1.POSITION: `PositionRetireOrSuspendCommitRequest` | PH1.POSITION | FOUND | Active/Suspended -> Retired. |
| position requirement schema | PH1.POSITION contract/storage | PH1.POSITION | FOUND | Versioned schema create/update/activate. |
| position required fields | `PositionRequirementFieldSpec` | PH1.POSITION | FOUND | Field key/type/rule/sensitivity/exposure/evidence/prompts. |
| position hierarchy | no manager/reports-to graph found | PH1.POSITION | NOT_FOUND | DESIGN_GAP for hierarchy/reporting line. |
| job family | `PositionSchemaSelectorSnapshot.position_family` | PH1.POSITION | PARTIAL | Selector exists; no registry/hierarchy. |
| department | `PositionRecord.department` | PH1.POSITION for position attribute; org owner future for hierarchy | FOUND/PARTIAL | String field, no department registry. |
| manager/reporting line | no first-class field found | PH1.POSITION / Org owner | NOT_FOUND | DESIGN_GAP. |
| company/tenant scope | `TenantId`, `company_id`, `TenantCompanyRecord` | Tenant/Company owner plus PH1.POSITION scoped relation | FOUND | Position is tenant/company scoped. |
| workspace scope | no `workspace_id` in `PositionRecord` | Workspace Governance + PH1.POSITION relation | NOT_FOUND | SECURITY_GAP for workspace-specific positions. |
| country/region/industry/company-size overlays | `jurisdiction`; selectors `company_size`, `industry_code`, `jurisdiction`, `position_family` | PH1.POSITION + overlay governance | PARTIAL | Selector snapshot exists; full overlay engine not proven. |
| salary/pay band reference | `compensation_band_ref`, `PositionBandPolicyCheckRequest` | Payroll/HR owns pay truth; PH1.POSITION may reference band | PARTIAL | Band ref checked for equality; no salary/payroll truth. |
| payroll group reference | not found | Payroll/HR owner | NOT_FOUND | DESIGN_GAP. |
| roster/schedule group reference | `PositionScheduleType` only | Scheduler/Roster owner | PARTIAL | Schedule type enum exists; no roster group truth. |
| training/certification requirements | `field_specs` can represent fields; no training owner | PH1.POSITION requirement refs + Training owner | PARTIAL | Can model via field key/evidence, but no training lifecycle. |
| document/photo proof requirements | `PositionRequirementEvidenceMode::DocRequired`; ONB maps to photo/sender gates | PH1.POSITION defines requirement; Document/Artifact and ONB handle proof flow | FOUND/PARTIAL | Current ONB derives gates from DocRequired. |
| onboarding required fields | `ph1link_schema_required_fields`, active schema current | PH1.POSITION schema consumed by PH1.ONB | FOUND | ONB read-only consumption. |
| access template mapping | `permission_profile_ref`; Access compile `position_id` lineage | Access/Governance owns permissions; PH1.POSITION may reference candidate profile | PARTIAL | Must not be treated as access grant. |
| role/access candidate mapping | no full role template registry in Position | Access/Governance + PH1.POSITION relation | PARTIAL | Current field is reference string. |
| promotion/demotion relation | no current position-change flow found | PH1.POSITION + Access + HR | NOT_FOUND | DESIGN_GAP. |
| termination/resignation relation | no current position termination flow found | HR/Payroll + PH1.POSITION + Access | NOT_FOUND | DESIGN_GAP. |
| Desktop rendering | none found | Desktop render-only | NOT_FOUND | No direct UI. |
| iPhone rendering | none found | iPhone render-only | NOT_FOUND | No direct UI. |
| Adapter transport | no direct position routes; seeded fixtures only | Adapter transport-only | PARTIAL | Adapter must not own position truth. |
| audit/provenance | PH1.J state transition audit in `Ph1PositionRuntime`, lifecycle event records | PH1.POSITION + PH1.J | FOUND | Runtime emits PH1.J; storage ledger has simulation/reason/actor/idempotency. |
| storage/migrations | PH1.F storage; SQL migration for schema/backfill | PH1.POSITION storage owner | PARTIAL | Direct SQL table evidence incomplete for position rows/lifecycle events. |
| old compatibility paths | historical packet docs; seeded adapter/onboarding paths | proof-based retirement ledger | PARTIAL | No deletion in this task. |

## 4. Current Position Lifecycle

### position draft created

- owner: PH1.POSITION.
- symbols/files: `PositionCreateDraftRequest`, `PositionRequest::CreateDraft`, `Ph1PositionRuntime::run`, `ph1position_create_draft`, `PH1POSITION_CREATE_DRAFT_ROW`.
- inputs: actor user id, tenant id, company id, position title, department, jurisdiction, schedule type, permission profile ref, compensation band ref, idempotency key, simulation id.
- outputs: `PositionCreateDraftResult` with `position_id` and `lifecycle_state=Draft`.
- state changes: inserts `PositionRecord` keyed by tenant and position id; appends `PositionLifecycleEventRecord`.
- audit evidence: PH1.J state transition `NONE -> DRAFT_CREATED`; lifecycle event action `CreateDraft`.
- gaps: no direct SQL migration for `positions` table found; no workspace id; no position hierarchy.

### required fields defined

- owner: PH1.POSITION.
- symbols/files: `PositionRequirementFieldSpec`, `PositionRequirementsSchemaCreateDraftRequest`, `ph1position_requirements_schema_create_draft`.
- inputs: tenant, company, position, schema version id, selector snapshot, field specs, idempotency key.
- outputs: `PositionRequirementsSchemaDraftResult` with field count.
- state changes: appends `PositionRequirementsSchemaLedgerRecord` action `CreateDraft`.
- audit evidence: PH1.J state transition `REQUIREMENTS_SCHEMA_NONE -> REQUIREMENTS_SCHEMA_DRAFT_CREATED`; ledger event has actor/reason/idempotency.
- gaps: no sender-prefill vs receiver-provided flag; no one-time field override; no field owner registry.

### schema version created

- owner: PH1.POSITION.
- symbols/files: `position_requirements_schema_ledger`, `PositionRequirementsSchemaLedgerAction::CreateDraft`.
- inputs: schema version id and field specs.
- outputs: draft schema result.
- state changes: append-only schema ledger.
- audit evidence: ledger record and PH1.J.
- gaps: no explicit schema status enum beyond ledger action/current pointer.

### position activated

- owner: PH1.POSITION.
- symbols/files: `PositionActivateCommitRequest`, `ph1position_activate_commit`.
- inputs: tenant, position, actor, idempotency key, commit simulation.
- outputs: lifecycle result `Active`.
- state changes: Draft/Suspended -> Active; append lifecycle event.
- audit evidence: PH1.J `DRAFT_OR_SUSPENDED -> ACTIVE`; lifecycle event action `Activate`.
- gaps: blueprint requires confirmation/access gate, but direct runtime does not itself resolve human confirmation; it expects caller/orchestration to gate.

### position updated

- owner: PH1.POSITION for requirements schema updates.
- symbols/files: `PositionRequirementsSchemaUpdateCommitRequest`, `change_reason`, `ph1position_requirements_schema_update_commit`.
- inputs: active position, schema version id, selectors, field specs, change reason.
- outputs: schema draft result with updated field count.
- state changes: appends schema ledger action `UpdateCommit`.
- audit evidence: PH1.J `REQUIREMENTS_SCHEMA_DRAFT_OR_ACTIVE -> REQUIREMENTS_SCHEMA_UPDATED`.
- gaps: no current general position metadata update for title/department/jurisdiction/schedule type/profile refs.

### position retired

- owner: PH1.POSITION.
- symbols/files: `PositionRetireOrSuspendCommitRequest`, `ph1position_retire_or_suspend_commit`.
- inputs: tenant, position, requested state Suspended or Retired, actor, idempotency key.
- outputs: lifecycle result.
- state changes: Active/Suspended -> Suspended or Retired.
- audit evidence: PH1.J `ACTIVE_OR_SUSPENDED -> SUSPENDED/RETIRED`; lifecycle event.
- gaps: no downstream Access/ONB/Scheduler/HR handoff behavior proven for retirement.

### position linked to tenant/company/workspace

- owner: PH1.POSITION for current tenant/company relation.
- symbols/files: `TenantId`, `company_id`, `TenantCompanyRecord`, `ph1tenant_company_upsert`.
- inputs: tenant/company refs.
- outputs: position records scoped to tenant/company.
- state changes: none beyond position row.
- audit evidence: position lifecycle event.
- gaps: workspace scope NOT_FOUND.

### position linked to onboarding requirement schema

- owner: PH1.POSITION schema; PH1.ONB consumption.
- symbols/files: `position_requirements_schema_current`, `ph1link_schema_required_fields`, `ph1onb_session_start_draft_row`.
- inputs: active schema for employee invite prefilled `tenant_id` and `position_id`.
- outputs: required field list, pinned schema id/version/overlay selector ref.
- state changes: ONB session pins refs and required gates.
- audit evidence: ONB session records and PH1.POSITION schema ledger.
- gaps: limited to employee flows; no full universal onboarding type matrix in current code.

### position linked to access template

- owner: Access/Governance, not PH1.POSITION.
- symbols/files: `PositionRecord.permission_profile_ref`, `AccessCompiledLineageRef.position_id`, `ph1access_instance_compile_commit`.
- inputs: position ref or permission profile ref.
- outputs: access compiled lineage can include position id.
- state changes: access instance compile state, not position state.
- audit evidence: Access audit/records in Access stack.
- gaps: no complete template co-authoring, propagation, or Access recompile from position update proven.

### position linked to payroll/roster/scheduler fields

- owner: Payroll/HR and Scheduler/Roster future owners.
- symbols/files: `compensation_band_ref`, `PositionScheduleType`.
- inputs: compensation band ref and schedule type.
- outputs: policy check result and stored enum.
- state changes: position record only.
- audit evidence: position lifecycle events.
- gaps: payroll group, pay rate, roster group, shift pattern, leave/off-shift posture NOT_FOUND.

### position used during onboarding

- owner: PH1.ONB consumes; PH1.POSITION owns schema.
- symbols/files: `ph1onb_validate_employee_position_prereq`, `ph1onb_required_verification_gates_for_token`, `at_position_db_06_onb_read_only_schema_boundary`.
- inputs: link prefilled context with tenant/company/position/compensation tier.
- outputs: validation or refusal; required gates and fields.
- state changes: onboarding session and gate status, not position.
- audit evidence: ONB tests and records.
- gaps: no PH1.WRITE guidance; no full dynamic field explanation path.

### position used during access compile

- owner: Access/Governance.
- symbols/files: `AccessCompiledLineageRef.position_id`, `ph1access_instance_compile_commit`.
- inputs: optional position id.
- outputs: access instance `compiled_position_id`.
- state changes: access instance.
- audit evidence: Access stack evidence.
- gaps: position id existence check only; no permission derivation from position is proven in current Position code.

### position changed during promotion/demotion

- owner: future PH1.POSITION + HR + Access.
- symbols/files: none found for promotion/demotion relation in current PH1.POSITION.
- status: NOT_FOUND.
- gaps: DESIGN_GAP, OWNER_GAP.

## 5. Data Model / Contracts / Packets

### Request structs

| Name | Evidence | Status | Notes |
|---|---|---|---|
| `Ph1PositionRequest` | `crates/selene_kernel_contracts/src/ph1position.rs` | FOUND | Wrapper with schema version, correlation id, turn id, now, simulation id/type, request variant. |
| `PositionCreateDraftRequest` | same | FOUND | actor, tenant, company, title, department, jurisdiction, schedule type, permission profile ref, compensation band ref, idempotency. |
| `PositionValidateAuthCompanyRequest` | same | FOUND | actor, tenant, company, position, requested action, idempotency. |
| `PositionBandPolicyCheckRequest` | same | FOUND | actor, tenant, position, compensation band ref, idempotency. |
| `PositionActivateCommitRequest` | same | FOUND | actor, tenant, position, idempotency. |
| `PositionRetireOrSuspendCommitRequest` | same | FOUND | actor, tenant, position, requested state, idempotency. |
| `PositionRequirementsSchemaCreateDraftRequest` | same | FOUND | actor, tenant, company, position, schema version, selectors, field specs, idempotency. |
| `PositionRequirementsSchemaUpdateCommitRequest` | same | FOUND | create fields plus `change_reason`. |
| `PositionRequirementsSchemaActivateCommitRequest` | same | FOUND | actor, tenant, company, position, schema version, apply scope, idempotency. |
| `PositionCreationJourneyRequestPacket` | future equivalent only | NOT_FOUND | No such named packet in repo truth. |

### Response structs

| Name | Evidence | Status | Notes |
|---|---|---|---|
| `Ph1PositionResponse` | kernel contract | FOUND | `Ok` or `Refuse`. |
| `Ph1PositionOk` | kernel contract | FOUND | Holds simulation id, reason code, optional result payloads. |
| `Ph1PositionRefuse` | kernel contract | FOUND | Refusal path exists. |
| `PositionCreateDraftResult` | kernel contract | FOUND | position id + lifecycle state. |
| `PositionValidateAuthCompanyResult` | kernel contract | FOUND | validation status + reason code. |
| `PositionBandPolicyCheckResult` | kernel contract | FOUND | policy result + reason code. |
| `PositionLifecycleResult` | kernel contract | FOUND | position id + lifecycle state. |
| `PositionRequirementsSchemaDraftResult` | kernel contract | FOUND | position id + schema version id + field count. |
| `PositionRequirementsSchemaLifecycleResult` | kernel contract | FOUND | position id, schema version id, apply scope result, backfill handoff flag. |

### Records

| Name | Evidence | Status | Notes |
|---|---|---|---|
| `TenantCompanyRecord` | `crates/selene_storage/src/ph1f.rs` | FOUND | PH1.POSITION ECM includes scoped company write/read. |
| `PositionRecord` | kernel contract | FOUND | tenant/company/title/department/jurisdiction/schedule/profile ref/band ref/lifecycle/timestamps. |
| `PositionLifecycleEventRecord` | PH1.F | FOUND | append-only event record with action, states, reason, simulation, actor, idempotency. |
| `PositionRequirementsSchemaLedgerRecord` | PH1.F | FOUND | schema event ledger with selectors, field specs, reason, apply scope. |
| `PositionRequirementsSchemaCurrentRecord` | PH1.F | FOUND | active schema pointer and field specs. |
| `OnbRequirementBackfillCampaignRecord` | PH1.F and migration | FOUND | ONB backfill campaign relation for CurrentAndNew schema activation. |
| `OnbRequirementBackfillTargetRecord` | migration and ONB storage | FOUND | Backfill target state exists in ONB domain. |

### Enums and status states

| Name | Values / Fields | Status | Notes |
|---|---|---|---|
| `PositionLifecycleState` | Draft, Active, Suspended, Retired | FOUND | Core position state. |
| `PositionLifecycleAction` | CreateDraft, Activate, Suspend, Retire, PolicyOverride | FOUND | Event action. |
| `PositionRequestedAction` | Activate, Suspend, Retire | FOUND | Validation request action. |
| `PositionValidationStatus` | Ok, Fail | FOUND | Validation result. |
| `PositionPolicyResult` | Allow, Escalate, Deny | FOUND | Band policy result. |
| `PositionScheduleType` | FullTime, PartTime, Contract, Shift | FOUND | Position attribute only. |
| `PositionRequirementFieldType` | String, Integer, Decimal, Date, Enum, Object | FOUND | Requirement field type. |
| `PositionRequirementRuleType` | Always, Conditional | FOUND | Required-field rule. |
| `PositionRequirementSensitivity` | Safe, Private, Confidential | FOUND | Sensitivity flag. |
| `PositionRequirementExposureRule` | Speak, TextOnly, InternalOnly | FOUND | Presentation/exposure hint. |
| `PositionRequirementEvidenceMode` | UserAnswer, DocRequired, ToolDerived, Attestation | FOUND | Evidence mode used by ONB gate derivation. |
| `PositionSchemaApplyScope` | NewHiresOnly, CurrentAndNew | FOUND | Backfill handoff semantics. |

### Migration tables

| Table | Evidence | Status | Notes |
|---|---|---|---|
| `position_requirements_schema_ledger` | migration 0014 | FOUND | Append-like SQL schema for requirements events. |
| `position_requirements_schema_current` | migration 0014 | FOUND | Active schema pointer. |
| `onboarding_requirement_backfill_campaigns` | migration 0014 | FOUND | ONB campaign records. |
| `onboarding_requirement_backfill_targets` | migration 0014 | FOUND | ONB target records. |
| `positions` | DB wiring/PH1.F | PARTIAL | Runtime/storage evidence exists; direct SQL create not found in migrations searched. |
| `position_lifecycle_events` | DB wiring/PH1.F | PARTIAL | Runtime/storage evidence exists; direct SQL create not found in migrations searched. |

### IDs and refs

| ID / Ref | Evidence | Status | Notes |
|---|---|---|---|
| `TenantId` | kernel contract | FOUND | Validated id wrapper. |
| `PositionId` | kernel contract | FOUND | Deterministically generated in PH1.F using hash. |
| `company_id` | `PositionRecord` | FOUND | String ref; company must be active. |
| `workspace_id` | search | NOT_FOUND | No PositionRecord workspace field. |
| `schema_version_id` | schema requests/current/ledger | FOUND | Active schema version pointer. |
| `permission_profile_ref` | `PositionRecord` | FOUND | Reference only; not access grant. |
| `compensation_band_ref` | `PositionRecord` | FOUND | Reference plus band policy check. |
| `role_id` / `role_template_id` | PH1.POSITION | NOT_FOUND | Belongs to Access in current evidence, not PH1.POSITION. |
| `department_id` | PH1.POSITION | NOT_FOUND | Department is string. |
| `payroll_group_id` | PH1.POSITION | NOT_FOUND | Payroll owner gap. |
| `roster_group_id` | PH1.POSITION | NOT_FOUND | Scheduler/Roster owner gap. |

### Audit/provenance fields

| Field | Evidence | Status | Notes |
|---|---|---|---|
| `simulation_id` | lifecycle events/schema records | FOUND | Stored with position lifecycle/schema changes. |
| `reason_code` | lifecycle events/schema records/runtime | FOUND | Reason-coded operations. |
| `actor_user_id` | lifecycle/schema requests and records | FOUND | Actor present. |
| `idempotency_key` | requests/lifecycle/schema records | FOUND | Idempotency indexes and replay semantics. |
| `correlation_id` / `turn_id` | `Ph1PositionRequest`, PH1.J audit | FOUND | Runtime audit context. |
| `created_at` / `updated_at` | records | FOUND | Monotonic timestamps. |
| client/device refs | direct PH1.POSITION | NOT_FOUND | Position audit uses PH1.J with no direct device/session fields in PH1.POSITION event. |

## 6. Position Types And Product Functions

| Type / Product Function | Evidence Found | Current Behavior | Owner | Security Implications | Missing Design |
|---|---|---|---|---|---|
| employee position | Employee invite/ONB uses `position_id`; tests use Driver/Manager/Supervisor | Active position can drive employee ONB fields/gates | PH1.POSITION + PH1.ONB | Position must be active and company-scoped | full employee position lifecycle and HR integration |
| contractor position | `PositionScheduleType::Contract` | Can store schedule type Contract | PH1.POSITION | Does not prove contractor onboarding policy | contractor-specific schemas |
| executive position | examples in future docs only | NOT_FOUND as specific logic | PH1.POSITION + Access | High-access risk not modeled | executive role/approval policy |
| manager position | tests use Manager/Store Manager/Warehouse Manager strings | Generic title string | PH1.POSITION | No hierarchy/authority from title | role hierarchy and access mapping |
| warehouse position | tests use Warehouse/Driver/Stock Lead | Generic title/department/family selector | PH1.POSITION | No warehouse-specific roster/access proof | warehouse requirement packs |
| retail position | tests and selector use Retail | Generic department/industry selector | PH1.POSITION | No retail-specific policy | retail requirement packs |
| sales position | search did not find current position-specific sales logic | NOT_FOUND | PH1.POSITION future | unknown | sales requirement pack |
| finance position | future access docs mention CFO/finance; PH1.POSITION direct logic not found | NOT_FOUND | PH1.POSITION + Access/Finance | high risk | finance requirement/access co-authoring |
| HR position | direct PH1.POSITION logic not found | NOT_FOUND | PH1.POSITION + HR/Access | high risk | HR requirement/access co-authoring |
| admin/system position | direct PH1.POSITION logic not found | NOT_FOUND | PH1.POSITION + Access | high risk | no title-as-permission |
| customer-facing role | no PH1.POSITION-specific behavior | NOT_FOUND | Position/CRM future | privacy risk | customer role matrix |
| supplier-facing role | no PH1.POSITION-specific behavior | NOT_FOUND | Position/Supplier future | supplier bank data risk | supplier role matrix |
| workspace role | no workspace field | NOT_FOUND | Workspace Governance + Access | scope risk | workspace scope |
| department role | `department` string exists | PARTIAL | PH1.POSITION | no hierarchy | department registry |
| company/tenant role | tenant/company scope exists | PARTIAL | PH1.POSITION + Tenant/Governance | tenant isolation enforced in tests | company/entity role model |
| temporary/casual role | no position lifecycle relation beyond schedule type | NOT_FOUND/PARTIAL | PH1.POSITION + HR/Access | access duration risk | temporary position/contract term |
| trainee/apprentice role | not found | NOT_FOUND | PH1.POSITION future | training/compliance risk | trainee requirement pack |
| custom position | arbitrary title accepted | FOUND | PH1.POSITION | could create uncontrolled titles if gate weak | canonical review/approval |
| retired position | `PositionLifecycleState::Retired` | FOUND | PH1.POSITION | downstream consumers must not use retired positions | downstream retirement handoffs |

## 7. Position Requirement Schema Logic

Position requirement schemas are FOUND.

Evidence:

- `PositionRequirementFieldSpec`
- `PositionSchemaSelectorSnapshot`
- `PositionRequirementsSchemaCreateDraftRequest`
- `PositionRequirementsSchemaUpdateCommitRequest`
- `PositionRequirementsSchemaActivateCommitRequest`
- `PositionRequirementsSchemaLedgerRecord`
- `PositionRequirementsSchemaCurrentRecord`
- migration `0014_position_requirements_schema_and_backfill_tables.sql`
- `docs/BLUEPRINTS/ONB_SCHEMA_MANAGE.md`
- storage test `at_position_db_05_requirements_schema_activation_monotonic`

Schemas are versioned by `schema_version_id`, represented as append records in `position_requirements_schema_ledger`, and activated through `position_requirements_schema_current`. Activation is monotonic in the sense that current pointer moves to the activated schema version, activation requires a prior create/update ledger event, and idempotency replays preserve the originally selected `PositionSchemaApplyScope`.

Required fields are tied to position through `(tenant_id, position_id)` and schema records. They are also tied to company through `company_id` and position company validation.

Required fields can be conditional on selector snapshot keys. Current allowed selector keys are `company_size`, `industry_code`, `jurisdiction`, and `position_family`. A conditional predicate must use a bounded selector expression such as `selector.jurisdiction=US`; unsupported selector keys fail with contract violation.

Schemas are activated, not silently mutated. `PositionRequirementsSchemaUpdateCommitRequest` requires `change_reason`. `PositionRequirementsSchemaActivateCommitRequest` uses `apply_scope`.

Existing onboarding sessions are protected by pinned schema/selector refs. ONB code comments state pinned session gates are authoritative for replay stability and should not reinterpret required gates from current schema state once a session has started.

Requirement backfill is supported as a handoff, not as PH1.POSITION doing ONB work. `PositionSchemaApplyScope::CurrentAndNew` produces `backfill_handoff_required=true`. `ONB_SCHEMA_MANAGE` then routes to PH1.ONB backfill start for current-and-new rollout.

Sender-prefill vs receiver-provided fields are PARTIAL. Current repo has `PrefilledContext` fields from PH1.LINK and required field specs, but `PositionRequirementFieldSpec` does not explicitly declare `sender_prefill_allowed` or `receiver_input_allowed`.

One-time field overrides are NOT_FOUND in PH1.POSITION current repo truth.

Missing:

- formal field owner registry.
- schema-level approval workflow completion.
- one-time override.
- workspace overlay.
- country/region/industry/company-size overlay lifecycle beyond selector snapshot.
- field sensitivity access policy enforcement beyond stored sensitivity/exposure/evidence flags.

## 8. Dynamic Field Requirements And Overlays

Current support is PARTIAL.

PH1.POSITION can model dynamic required fields as field specs:

- `field_key`
- `field_type`
- `required_rule`
- `required_predicate_ref`
- `validation_ref`
- `sensitivity`
- `exposure_rule`
- `evidence_mode`
- `prompt_short`
- `prompt_long`

Examples that can be represented as current field specs:

- `superannuation_member_number` as a private string field.
- `health_number` as confidential/private string field.
- `tax_number` as private string field.
- `bank_details` as private object field.
- `forklift_licence_number` as private string field with doc evidence.
- `safety_certification` as doc-required field.
- `emergency_contact` as object/string field.
- `payroll_group` or `roster_group` as string fields.
- `commission_plan` as string/object field.

However, repo truth does not prove full governance for these examples. It only proves that fields can be encoded, validated structurally, stored in schema ledger/current, and consumed by ONB for required-field/gate derivation.

| Capability | Status | Evidence | Notes |
|---|---|---|---|
| add field for one person only | NOT_FOUND | no one-time field packet/storage | Must not be invented. |
| add field to position permanently | PARTIAL | schema update + activate for position | Requires commit, change reason, active position; no full approval workflow in PH1.POSITION runtime. |
| add field to tenant/company | PARTIAL | company id in schema records; selectors | No tenant-wide overlay engine proven. |
| add field by country/region | PARTIAL | selector `jurisdiction` | Selector only, not authoritative country policy owner. |
| add field by industry | PARTIAL | selector `industry_code` | Selector only, no industry overlay owner. |
| add field globally/default | NOT_FOUND | no global position schema in PH1.POSITION | DESIGN_GAP. |
| version changes | FOUND | ledger/current + schema version id | Versioning exists. |
| sensitivity classification | FOUND/PARTIAL | `PositionRequirementSensitivity` | Stored, but downstream access enforcement is not fully proven. |
| field access/edit permissions | NOT_FOUND in PH1.POSITION | Access field-level model future | SECURITY_GAP. |

## 9. Position Hierarchy And Reporting Line

Position hierarchy is NOT_FOUND as a first-class current repo model.

Evidence found:

- `PositionRecord.position_title`
- `PositionRecord.department`
- `PositionSchemaSelectorSnapshot.position_family`
- arbitrary test titles such as Store Manager, Warehouse Manager, Warehouse Supervisor, Driver, Stock Lead.

Evidence not found:

- `reports_to`
- `manager_position_id`
- `supervisor_position_id`
- org chart tables.
- department hierarchy tables.
- role seniority enum.
- promotion/demotion relation.
- who-can-manage-whom logic inside PH1.POSITION.
- position hierarchy linked to access authority.

Current conclusion:

PH1.POSITION can name and scope a position; it does not currently prove a hierarchical org model. Role hierarchy and access hierarchy belong to Access/Governance and future PH1.POSITION reconciliation, not to string titles.

## 10. Position-To-Access Relationship

The current relationship is PARTIAL.

Evidence:

- `PositionRecord.permission_profile_ref` stores a permission profile reference string.
- `AccessCompiledLineageRef` in `ph1access.rs` includes optional `position_id`.
- `ph1access_instance_compile_commit` validates that referenced `position_id` exists in `positions` for the tenant.
- `AccessInstanceRecord` stores `compiled_position_id`.
- `docs/BLUEPRINTS/ACCESS_INSTANCE_COMPILE_REFRESH.md` includes optional `position_id` in `ACCESS_READ_SCHEMA_CHAIN_ROW`.
- Master Access repo-truth extraction identifies position binding required for compile tests.

What this proves:

- Access can bind compiled lineage to a current position id.
- Position can carry a permission profile reference.
- Access compile can fail if a referenced position does not exist.

What this does not prove:

- a full access template registry owned by PH1.POSITION.
- permission derivation directly from position.
- automatic access recompile when position changes.
- promotion/demotion access change.
- Access template co-authoring.
- template propagation to per-user access.

Critical current rule:

Position defines the job. Access grants the permissions. PH1.POSITION must not grant access directly.

## 11. Position-To-Onboarding Relationship

The current relationship is FOUND for employee onboarding and PARTIAL for broader onboarding.

Evidence:

- `PrefilledContext.position_id` appears in PH1.LINK and ONB flows.
- `ph1link_schema_required_fields` prefers active tenant position schema for employee flows.
- fallback employee fields are `company_id`, `position_id`, `location_id`, `start_date` when no active schema can be resolved.
- `ph1onb_validate_employee_position_prereq` validates tenant/company/position existence, active company, active position, and optional compensation tier match.
- session start can pin schema id, active schema version, overlay set id, and selector snapshot ref when employee link has active position schema.
- `ph1onb_required_verification_gates_for_token` maps DocRequired fields to `PHOTO_EVIDENCE_CAPTURE` and `SENDER_CONFIRMATION`.
- storage test `at_position_db_06_onb_read_only_schema_boundary` proves ONB cannot create access before required position-derived verification gates are satisfied.

Current answers:

- Does onboarding load position requirements? FOUND for employee link/session flow.
- Does position determine missing fields? FOUND/PARTIAL; required fields are computed from active schema, with fallback template.
- Does position determine employee photo/sender verification gates? FOUND when field spec uses `DocRequired`.
- Does position determine sender prefill fields? PARTIAL; link `PrefilledContext` exists, but Position schema does not classify sender vs receiver fields.
- Does position determine receiver-provided fields? PARTIAL; required field list exists, but receiver ownership is not explicit.
- Does position determine voice/device/document requirements? PARTIAL; DocRequired maps to photo/sender gates. Voice/device gates are ONB/Voice/Device owners.
- Does position determine salary/start date/department/region fields? PARTIAL; department/jurisdiction/band ref exist, start date is link prefill, salary/payroll truth is not Position-owned.
- Does position determine backfill requirements? FOUND through schema apply scope and ONB backfill handoff.

Missing:

- universal onboarding target model in runtime.
- PH1.WRITE guidance for field explanations.
- one-time field override.
- explicit field requester/source owner policy.

## 12. Position-To-Payroll / HR Relationship

The relationship is PARTIAL and reference-only.

Current evidence:

- `PositionRecord.compensation_band_ref`.
- `PositionBandPolicyCheckRequest`.
- `ph1position_band_policy_check_draft`.
- ONB position prereq checks that `PrefilledContext.compensation_tier_ref` matches `position.compensation_band_ref` when present.
- docs and kernel contract state that raw salary values must not be embedded in link tokens.

Not found:

- salary amount.
- pay rate.
- currency.
- pay frequency.
- benefits.
- superannuation owner.
- tax details owner.
- health number owner.
- payroll group.
- HR documents owner.
- HR employment status.
- probation state.
- termination/resignation/retirement flow.

Critical rule:

Position may reference compensation band or requirement fields, but Payroll/HR must own payroll and employment truth. PH1.POSITION does not currently prove Payroll/HR implementation.

## 13. Position-To-Scheduler / Roster / Workload Relationship

The relationship is PARTIAL.

Current evidence:

- `PositionScheduleType::{FullTime, PartTime, Contract, Shift}`.
- tests use FullTime, PartTime, Shift.
- onboarding fallback fields include `location_id` and `start_date`, not roster truth.

Not found:

- roster group.
- shift pattern.
- availability.
- schedule group.
- workload owner.
- work location registry.
- department schedule.
- leave/off-shift access posture owned by Position.

Critical rule:

Position may define expected schedule shape or references. Scheduler/Roster owners must own scheduling truth. PH1.POSITION must not mutate rosters, schedules, leave, or workload from its own lifecycle.

## 14. Position Creation + Access Template Co-Authoring

The desired future flow is PARTIAL in current repo truth.

Current support:

- `POSITION_MANAGE` blueprint has a guided lifecycle orchestration using PH1.C, PH1.NLP, PH1.X, Access gate, and PH1.POSITION.
- `PositionRecord.permission_profile_ref` can store a permission profile reference.
- Access compile can validate a `position_id`.
- Master Access Journey design describes future co-authoring.

Current not supported:

- no runtime one-shot flow for "Create Warehouse Manager position" with GPT-5.5 proposal.
- no PH1.POSITION-owned position details wizard.
- no access template proposal packet.
- no authorized access-template review stored by Position.
- no PH1.POSITION activation tied to Access template activation.
- no future onboarding usage bundle that loads position + access template as one coherent journey.

Current owner split:

- PH1.POSITION stores position lifecycle and position requirement schema.
- Access/Governance stores permissions and access templates.
- PH1.ONB consumes position requirements during onboarding.
- Current co-authoring remains a future design need.

## 15. PH1.D / GPT-5.5 / PH1.N / PH1.X Interaction

Current probabilistic support is PARTIAL and mostly blueprint-level.

Evidence:

- `docs/BLUEPRINTS/POSITION_MANAGE.md` routes through PH1.C transcript, PH1.NLP intent draft, PH1.X confirmation, Access gate, then PH1.POSITION.
- `docs/BLUEPRINTS/ONB_SCHEMA_MANAGE.md` routes through PH1.C, PH1.NLP, PH1.X clarification/confirmation, Access gate, PH1.POSITION schema steps, and optional PH1.ONB backfill.
- `crates/selene_os/src/simulation_executor.rs` can execute PH1.POSITION requests and ONB position live sequence.

Not found:

- current GPT/OpenAI provider call for position design.
- current PH1.D provider-governed position proposal packet.
- current PH1.N role/position extraction implementation specific to PH1.POSITION.
- current PH1.X position risk lane implemented as a complete runtime policy beyond blueprint orchestration.

Correct future rule:

OpenAI/GPT-5.5 may help propose position titles, requirements, field candidates, likely access-template candidates, and clarification questions. It must not create active positions, schemas, or permissions without deterministic PH1.X, Access, confirmation, simulation, PH1.POSITION, and audit gates.

## 16. PH1.WRITE Interaction

PH1.WRITE interaction is NOT_FOUND in current PH1.POSITION runtime.

Evidence found:

- `PositionRequirementFieldSpec.prompt_short` and `prompt_long` store prompt text for fields.
- Blueprints refer to PH1.X response/clarification steps.
- Runtime reason codes and audit transitions exist.

Evidence not found:

- PH1.WRITE-owned final position guidance.
- PH1.WRITE position denial/explanation boundary.
- PH1.WRITE review wording for position activation/retirement/schema changes.
- PH1.WRITE access-template review wording for position co-authoring.

Risks:

- POSITION_WRITING_OWNER_RISK: PARTIAL. Field prompts live in PH1.POSITION schema; final user-facing wording owner is not proven.
- HARDCODED_POSITION_GUIDANCE_RISK: PARTIAL. App ingress has hardcoded prompt text for missing `position_id`; schema field prompts are stored as text.
- CLIENT_POSITION_TEXT_RISK: NOT_FOUND for direct client position UI.
- ADAPTER_POSITION_TEXT_RISK: PARTIAL. Adapter surfaces onboarding routes and seeded tests; no direct position authoring route found.

Correct future rule:

PH1.WRITE owns final user-facing wording for position creation, review, denial, activation, access-template review, and onboarding requirement explanation.

## 17. Desktop / iPhone / Adapter Boundaries

Desktop:

- direct position UI evidence: NOT_FOUND.
- current client behavior: no direct PH1.POSITION render/create/update route found in `apple/mac_desktop`.
- risk: DESKTOP_POSITION_AUTHORITY_RISK is NOT_FOUND for direct position decisions, but future Desktop must remain render-only.

iPhone:

- direct position UI evidence: NOT_FOUND.
- current client behavior: no direct PH1.POSITION render/create/update route found in `apple/iphone`.
- risk: IPHONE_POSITION_AUTHORITY_RISK is NOT_FOUND for direct position decisions, but future iPhone must remain render-only.

Adapter:

- direct position routes: NOT_FOUND; route scan shows invite/onboarding/session routes, not `/position`.
- current behavior: adapter/lib and http adapter tests seed `PositionRecord` and active position schema for onboarding/invite tests.
- risk: ADAPTER_POSITION_AUTHORITY_RISK is PARTIAL because adapter fixtures create position records for tests, but no live position authority route was found. Future adapter must transport bounded requests only and must not decide position truth.

Runtime:

- PH1.POSITION runtime owns execution through `Ph1PositionRuntime::run`.
- Simulation executor dispatches PH1.POSITION.
- PH1.F stores current and ledger state.

## 18. Security / Privacy / Governance Model

Current evidence:

- actor identity must exist for create/activate/retire/schema operations.
- tenant and company must exist.
- company must be `TenantCompanyLifecycleState::Active`.
- positions are keyed by `(tenant_id, position_id)`.
- create uniqueness prevents duplicate `(tenant_id, company_id, position_title, department, jurisdiction)`.
- idempotency keys are required for retriable writes.
- lifecycle events are append-only.
- schema activation requires active position.
- schema update requires active position and `change_reason`.
- conditional requirements only allow bounded selector keys.
- field specs carry sensitivity and exposure rules.
- blueprints require Access gate and confirmation before commits.
- simulation catalog declares required roles and confirmations for commit operations.

Missing or partial:

- who can create/update/retire positions is not fully enforced inside PH1.POSITION runtime; blueprint says Access gate required.
- who can change position-to-access mappings is not fully modeled.
- who can view sensitive position fields is not enforced in PH1.POSITION.
- salary/pay band reference can be stored and checked, but Payroll/HR policy owner is missing.
- region/country overlay owner is partial.
- approval/escalation from band policy is only a result (`Escalate`) in storage; full approval completion is future Access/Governance.
- workspace scope is missing.
- field-level access permissions are missing.
- client/adapter render-only proof is missing for future position authoring UI.

Security gaps:

- SECURITY_GAP: position strings and permission refs must not become access authority.
- SECURITY_GAP: no workspace scope in `PositionRecord`.
- SECURITY_GAP: sensitive requirement fields are classified but not fully access-controlled in PH1.POSITION evidence.

## 19. Position State Machine

RECONSTRUCTED_FROM_REPO_EVIDENCE using actual states.

Actual states:

- Draft
- Active
- Suspended
- Retired

Transitions supported by PH1.F:

| From | Action | To | Evidence | Status |
|---|---|---|---|---|
| none | CreateDraft | Draft | `ph1position_create_draft` | FOUND |
| Draft | Activate | Active | `ph1position_activate_commit` | FOUND |
| Suspended | Activate | Active | `ph1position_activate_commit` | FOUND |
| Active | Suspend | Suspended | `ph1position_retire_or_suspend_commit` | FOUND |
| Active | Retire | Retired | `ph1position_retire_or_suspend_commit` | FOUND |
| Suspended | Retire | Retired | `ph1position_retire_or_suspend_commit` | FOUND |
| Draft | Suspend/Retire | invalid | storage contract violation | FOUND |
| Retired | Activate/Suspend/Retire | invalid | storage contract violation/validation fail | FOUND |

Possible future states from user prompt:

- PendingReview: NOT_FOUND.
- Superseded: NOT_FOUND.
- Deprecated: NOT_FOUND.
- Archived: NOT_FOUND.
- BackfillRequired: PARTIAL through schema activation `backfill_handoff_required`, not Position lifecycle state.
- BackfillRunning: belongs to PH1.ONB backfill, not Position lifecycle.
- BackfillComplete: belongs to PH1.ONB backfill.
- Failed: NOT_FOUND as position lifecycle state.

Schema state machine:

- no explicit schema status enum.
- ledger actions: CreateDraft, UpdateCommit, ActivateCommit, RetireCommit.
- current pointer exists after activate.
- apply scopes: NewHiresOnly and CurrentAndNew.

## 20. Error Handling And Reason Codes

Existing error/reason evidence:

| Error / Reason | Evidence | Status | Notes |
|---|---|---|---|
| position not found | `StorageError::ForeignKeyViolation { table: "positions.position_id" }` | FOUND | Used by lifecycle/schema/access compile checks. |
| tenant mismatch | tenant-keyed map; cross-tenant activation test fails | FOUND | `at_position_db_01_tenant_isolation_enforced`. |
| workspace mismatch | no workspace field | NOT_FOUND | DESIGN_GAP. |
| duplicate position | `StorageError::DuplicateKey` for tenant/company/title/department/jurisdiction | FOUND | Create draft duplicate guard. |
| inactive company | company must be Active | FOUND | Create/schema operations check company state. |
| inactive position | schema update/activate require Active | FOUND | contract violation. |
| retired position | lifecycle validations fail for invalid transitions | FOUND | no retired activation path. |
| schema missing | activate schema without draft/update returns FK violation | FOUND | `at_position_db_05`. |
| schema inactive | no explicit inactive schema status | PARTIAL | current pointer only after activation. |
| required field missing | field specs must be non-empty; ONB computes missing fields | PARTIAL | Specific ONB missing-field errors are outside Position. |
| invalid field | field spec validation | FOUND | conditional predicate and prompt constraints. |
| access template missing | Access compile schema ref missing, not Position | PARTIAL | Position does not validate `permission_profile_ref` against Access. |
| payroll/HR owner missing | no owner integration | NOT_FOUND | OWNER_GAP. |
| scheduler/roster owner missing | no owner integration | NOT_FOUND | OWNER_GAP. |
| permission denied | blueprint Access gate, not runtime-internal | PARTIAL | PH1.POSITION runtime trusts caller path. |
| approval required | `PositionPolicyResult::Escalate` and blueprint `ACCESS_AP_REQUIRED` | PARTIAL | Approval completion not in PH1.POSITION. |
| backfill required | `backfill_handoff_required` | FOUND | On CurrentAndNew schema activation. |
| unsupported position type | no type enum beyond schedule type | NOT_FOUND | DESIGN_GAP. |
| client route mismatch | no direct client route | NOT_FOUND | Future UI gap. |

Runtime reason codes:

- `POSITION_OK_CREATE_DRAFT`
- `POSITION_OK_VALIDATE_AUTH_COMPANY`
- `POSITION_OK_BAND_POLICY_CHECK`
- `POSITION_OK_ACTIVATE_COMMIT`
- `POSITION_OK_RETIRE_OR_SUSPEND_COMMIT`
- `POSITION_OK_REQUIREMENTS_SCHEMA_CREATE_DRAFT`
- `POSITION_OK_REQUIREMENTS_SCHEMA_UPDATE_COMMIT`
- `POSITION_OK_REQUIREMENTS_SCHEMA_ACTIVATE_COMMIT`

Storage validation reason examples:

- `0x5900_0102`: company not active.
- `0x5900_0103`: position company mismatch.
- `0x5900_0104`: invalid lifecycle action for state.
- `0x5900_0105`: compensation band mismatch causing escalation.

## 21. Audit / Provenance / Evidence

Current audit/provenance:

- position creation audited: FOUND through lifecycle event and PH1.J runtime audit.
- position update audited: PARTIAL; requirements schema updates audited, but general position metadata update does not exist.
- position retirement audited: FOUND through lifecycle event and PH1.J runtime audit.
- schema activation audited: FOUND through PH1.J and schema ledger/current records.
- requirement field change audited: FOUND through schema ledger with field specs, change reason, actor, reason code.
- access template mapping audited: PARTIAL; `permission_profile_ref` exists, but Access mapping/change audit is Access-owned and not proven as Position.
- onboarding usage audited: PARTIAL; ONB session records and tests prove consumption; direct PH1.POSITION audit of ONB usage not present.
- backfill audited: PARTIAL; migration/storage backfill records have campaign state; full backfill audit belongs to ONB.
- tenant/workspace/company refs recorded: tenant/company FOUND; workspace NOT_FOUND.
- country/industry overlay refs recorded: PARTIAL via selector snapshot.
- client/adapter position events audited: NOT_FOUND.

Audit fields present in records:

- `event_id` / `schema_event_id`
- `tenant_id`
- `company_id`
- `position_id`
- `action`
- `from_state` / `to_state`
- `reason_code`
- `simulation_id`
- `actor_user_id`
- `created_at`
- `updated_at`
- `idempotency_key`
- `change_reason`
- `apply_scope`

AUDIT_GAP:

- no proven client/device/session refs in PH1.POSITION audit event.
- no full Access-template mapping audit from Position.
- no position hierarchy audit because hierarchy is missing.

## 22. Current Tests / Smokes / Acceptance

| Test / Smoke | Path | What It Proves | What It Does Not Prove | Status |
|---|---|---|---|---|
| `ph1position_happy_path_create_validate_activate_suspend` | `crates/selene_os/src/ph1position.rs` | OS runtime can create, validate, policy-check, activate, and suspend. | SQL persistence, UI, Access integration. | FOUND |
| `ph1position_requirements_schema_create_update_activate_scope_outputs` | same | Runtime schema create/update/activate returns field count, apply scope, backfill flag, idempotent replay. | Full backfill execution or PH1.WRITE wording. | FOUND |
| `at_position_db_01_tenant_isolation_enforced` | `crates/selene_storage/tests/ph1_position/db_wiring.rs` | tenant isolation and cross-tenant activation failure. | Workspace scope. | FOUND |
| `at_position_db_02_append_only_enforced` | same | lifecycle events append-only guard. | External audit immutability. | FOUND |
| `at_position_db_03_idempotency_dedupe_works` | same | create/activate idempotency. | Distributed idempotency. | FOUND |
| `at_position_db_04_current_table_consistency_with_lifecycle_ledger` | same | current lifecycle state matches event ledger after suspend. | SQL-backed table state. | FOUND |
| `at_position_db_05_requirements_schema_activation_monotonic` | same | missing schema activation fails; schema v1/v2 activation; idempotency preserves apply scope; CurrentAndNew requires backfill. | Full approval workflow. | FOUND |
| `at_position_db_06_onb_read_only_schema_boundary` | same | ONB reads position schema, enforces DocRequired gates, cannot create access before verification. | Universal onboarding and full Access policy. | FOUND |
| `onb_live_position_sequence_runs_create_validate_policy_activate` | `crates/selene_os/src/ph1onb.rs` | ONB live sequence can invoke PH1.POSITION create/validate/policy/activate. | Full production UI or Access gate. | FOUND |
| `onb_live_position_sequence_skips_activate_when_policy_escalates` | same | Policy escalation prevents activation in ONB live sequence. | Approval resolution. | FOUND |
| `at_sim_exec_03_execute_position_create_draft_runs_ph1position_runtime` | `crates/selene_os/src/simulation_executor.rs` | Simulation executor dispatches PH1.POSITION runtime. | All position simulations. | FOUND |
| Adapter onboarding tests with seeded positions | `crates/selene_adapter/src/lib.rs`, `crates/selene_adapter/src/bin/http_adapter.rs` | Adapter/onboarding test fixtures can seed position and schema for invite/onboarding. | Direct Adapter position route or authority safety. | PARTIAL |

TEST_GAP:

- no direct Desktop/iPhone position UI tests found.
- no full position hierarchy tests.
- no workspace-scope tests.
- no payroll/roster owner integration tests.
- no PH1.WRITE position explanation tests.
- no GPT-5.5/PH1.D position proposal tests.
- no JD live acceptance test.

## 23. Old Paths / Compatibility / Wrong-Owner Risks

| Path / Symbol | Current Status | Correct Canonical Owner | Retirement Condition | Active-Caller Check Needed |
|---|---|---|---|---|
| role string used as position | PARTIAL risk; arbitrary `position_title` and `permission_profile_ref` strings exist | PH1.POSITION for position, Access for role permissions | explicit role/position registry and access templates | yes |
| access role template used as position | PARTIAL risk through `permission_profile_ref` | Access/Governance | Access template registry and Position-to-Access mapping proof | yes |
| onboarding role field used as position truth | PARTIAL; `PrefilledContext.position_id` is consumed | PH1.POSITION | ONB render-only/consume-only proof | yes |
| adapter position shortcuts | PARTIAL; tests seed position records | Adapter transport-only | prove no live Adapter position authority route | yes |
| client position choices deciding access | NOT_FOUND direct UI, future risk | Desktop/iPhone render-only, Access owner | UI render-only acceptance | yes |
| hardcoded position fields | PARTIAL; fallback employee fields, app ingress question for `position_id` | PH1.POSITION schema + PH1.WRITE | PH1.WRITE/field schema boundary | yes |
| position creating access directly | NOT_FOUND in PH1.POSITION; ONB access creation exists | Access/Governance | prove ONB/Position cannot grant policy | yes |
| access creating position directly | NOT_FOUND | PH1.POSITION | access compile only validates position id | yes |
| salary/payroll fields owned by position | PARTIAL risk via `compensation_band_ref` | Payroll/HR | Payroll/HR owner map and field classification | yes |
| scheduler/roster fields owned by position | PARTIAL risk via `PositionScheduleType` | Scheduler/Roster | Scheduler/Roster owner map | yes |
| stale docs/historical packets | FOUND | Grand Architecture reconciliation | reconcile and retire superseded packet language | yes |
| duplicate position/role engines | PARTIAL; no standalone `selene_engines` file, Access has role/access | PH1.POSITION + Access split | canonical owner proof | yes |

No old paths are deleted by this extraction.

## 24. Full Functionality Master List

| Functionality | Description | Evidence | Owner | Status | Future Action |
|---|---|---|---|---|---|
| create position draft | Create tenant/company-scoped position in Draft state. | `ph1position_create_draft` | PH1.POSITION | FOUND | Activation pack should preserve idempotency and audit. |
| validate authority/company | Validate active company, matching company/position, and lifecycle action. | `ph1position_validate_auth_company_draft` | PH1.POSITION + Access blueprint gate | FOUND/PARTIAL | Complete Access authority enforcement. |
| compensation band policy check | Compare requested band ref to stored position band ref. | `ph1position_band_policy_check_draft` | PH1.POSITION + Payroll/HR future | FOUND/PARTIAL | Replace equality-only check with governed Payroll/HR policy. |
| activate position | Draft/Suspended to Active. | `ph1position_activate_commit` | PH1.POSITION | FOUND | Add approval/confirmation proof. |
| suspend position | Active to Suspended. | `ph1position_retire_or_suspend_commit` | PH1.POSITION | FOUND | Add downstream Access/ONB handling. |
| retire position | Active/Suspended to Retired. | same | PH1.POSITION | FOUND | Add offboarding/Access/ONB handling. |
| append lifecycle event | Record state changes. | `PositionLifecycleEventRecord` | PH1.POSITION | FOUND | Add SQL persistence proof if required. |
| prevent lifecycle overwrite | Append-only guard. | `attempt_overwrite_position_lifecycle_event` | PH1.POSITION | FOUND | Preserve. |
| create requirements schema draft | Store field specs for a position schema. | `ph1position_requirements_schema_create_draft` | PH1.POSITION | FOUND | Add approval flow for sensitive fields. |
| update requirements schema | Update schema specs with change reason. | `ph1position_requirements_schema_update_commit` | PH1.POSITION | FOUND | Add scope/field governance. |
| activate requirements schema | Set active schema current pointer. | `ph1position_requirements_schema_activate_commit` | PH1.POSITION | FOUND | Add schema retirement/version migration proof. |
| backfill handoff flag | Signal when current sessions need backfill. | `PositionSchemaApplyScope::CurrentAndNew` | PH1.POSITION -> PH1.ONB | FOUND | Complete ONB backfill audit/progress. |
| conditional required fields | Evaluate selector predicate. | `ph1position_required_by_rule` | PH1.POSITION | FOUND | Expand/standardize overlay language. |
| load position requirements during onboarding | Employee onboarding reads active schema. | `ph1link_schema_required_fields` | PH1.ONB consumes PH1.POSITION | FOUND | Generalize to future onboarding targets. |
| pin schema on onboarding session | Session start pins active schema refs. | PH1.F ONB session start code | PH1.ONB | FOUND | Keep replay-stability law. |
| derive doc gates | DocRequired fields require photo/sender gates. | `ph1onb_required_verification_gates_for_token` | PH1.ONB + PH1.POSITION schema | FOUND/PARTIAL | Add document owner and consent model. |
| validate employee position prereq | Ensures company/position active and band match. | `ph1onb_validate_employee_position_prereq` | PH1.ONB consuming PH1.POSITION | FOUND | Expand wrong-role correction. |
| bind position to tenant/company | tenant id/company id in position record. | `PositionRecord` | PH1.POSITION | FOUND | Add workspace/company hierarchy. |
| bind position to department | string department field. | `PositionRecord.department` | PH1.POSITION | PARTIAL | Add department registry/hierarchy. |
| bind position to salary/pay band ref | stores `compensation_band_ref`. | `PositionRecord` | PH1.POSITION + Payroll/HR future | PARTIAL | Add Payroll/HR owner boundary. |
| bind position to roster group | not found. | none | Scheduler/Roster future | NOT_FOUND | Add boundary map. |
| position-to-access compile binding | Access lineage can include position id. | `AccessCompiledLineageRef.position_id` | Access/Governance | PARTIAL | Build co-authoring/recompile proof. |
| position hierarchy | not found. | none | PH1.POSITION future | NOT_FOUND | Add hierarchy/manager/reporting line. |
| promotion/demotion relation | not found. | none | PH1.POSITION + HR + Access | NOT_FOUND | Add lifecycle relation. |
| direct Desktop position UI | not found. | none | Desktop render-only future | NOT_FOUND | Build proof if UI added. |
| direct Adapter position route | not found. | routes scan | Adapter transport-only future | NOT_FOUND | Add route only after backend proof. |

## 25. Comparison To Master Architecture

PH1.ONB Onboarding Journey:

Current PH1.ONB already consumes PH1.POSITION for employee onboarding required fields, schema pinning, active company/position validation, document-derived gates, and requirement backfill handoff. The journey design's human-guided onboarding layer is future on top of this repo truth.

Master Access Governance + Per-User Access Journey:

Current Access can carry `position_id` in compiled lineage and validate that a position exists. Current PH1.POSITION has `permission_profile_ref`. This is a foundation for Position-to-Access co-authoring, not a completed implementation. Access remains permission owner.

Identity + Access + Authority Spine:

PH1.POSITION operations require actor identity and blueprint Access gate before protected commits. Authority/Simulation remain outside Position. Runtime PH1.POSITION validates contract/simulation id but does not become global authority.

PH1.D Proposal Gateway:

No current PH1.D position proposal path was found. Future PH1.D may propose position details/schema candidates only.

PH1.N Meaning Unravelling:

Blueprints use PH1.NLP intent draft. No current dedicated PH1.N position extraction path was found.

PH1.X Request Decision Lattice:

Blueprints require PH1.X confirmation/clarification and Access gate before commits. A full PH1.X runtime position-risk model remains future.

PH1.WRITE Human Presentation:

Current PH1.POSITION has stored prompts and reason codes, but PH1.WRITE final presentation boundary is missing.

Payroll/HR future owners:

Position stores compensation band reference only. Payroll/HR must own actual pay, salary, benefits, tax, employment status, and termination/resignation/retirement truth.

Scheduler/Roster future owners:

Position stores schedule type only. Scheduler/Roster must own shifts, rosters, availability, work allocation, leave/off-shift posture, and workload.

Tenant / Workspace Governance:

Tenant and company scope are present. Workspace scope is missing.

Desktop/iPhone render-only boundary:

No direct position UI found. Future UI must render and submit bounded actions only.

Adapter transport-only boundary:

No direct position route found. Adapter tests seed position state for onboarding. Future adapter route must transport only.

Old Compatibility Path Retirement:

Historical strict-fix plan docs and seeded test paths should be reconciled later, not deleted now.

## 26. Gaps / Missing Pieces

| Gap | Evidence | Risk | Recommended Future Action | Priority |
|---|---|---|---|---|
| missing standalone Position engine | no `crates/selene_engines/src/ph1position.rs` | runtime ownership may be unclear to future builders | Decide whether OS+PH1.F remains canonical or add engine wrapper intentionally. | Medium |
| missing position lifecycle update beyond activate/suspend/retire | no update request for title/department/etc. | stale or wrong position metadata | Add governed update/supersede path. | High |
| missing position hierarchy | no reports-to/manager fields | title strings may be misused for authority | Build hierarchy/reporting-line model. | High |
| missing department hierarchy | department string only | weak org governance | Add department registry owner or relation. | Medium |
| missing job family registry | selector only | inconsistent job families | Add job family registry/validation. | Medium |
| missing role/position distinction | `permission_profile_ref` and title strings | access drift from role names | Reconcile with Master Access role/template registry. | High |
| missing position-to-access template mapping | reference only | permissions may be guessed | Build explicit mapping and co-authoring. | High |
| missing position-to-onboarding schema mapping for all types | employee path only | non-employee onboarding inconsistent | Extend through ONB Journey after reconciliation. | Medium |
| missing dynamic field governance | no one-time/permanent scope choice | schema mutations from casual wording risk | Build dynamic field governance with Access approval. | High |
| missing country/industry/company-size overlay governance | selector snapshot only | compliance field errors by region/industry | Build overlay lifecycle and owner map. | High |
| missing salary/payroll/HR owner boundary implementation | band ref only | payroll data exposure or wrong owner mutation | Build Payroll/HR boundary and field classification. | High |
| missing scheduler/roster owner boundary implementation | schedule enum only | roster mutation from Position risk | Build Scheduler/Roster boundary map. | Medium |
| missing PH1.WRITE position guidance boundary | no PH1.WRITE evidence | hardcoded/confusing user guidance | Add PH1.WRITE position wording pack. | Medium |
| missing PH1.D/PH1.N position proposal path | blueprint only | fuzzy requests cannot be safely converted | Add provider-off/fake-provider proposal shell. | Medium |
| missing access co-authoring flow | future design only | Position/Access owners may merge unsafely | Build co-authoring with separate truth owners. | High |
| missing audit details for client/device/session | PH1.J audit no device/session | incomplete provenance for live UI | Add audit evidence pack if needed. | Medium |
| missing SQL persistence for position current/lifecycle | migration search only found schema/backfill tables | durable DB proof incomplete | Add/verify DB migration or document existing DB owner. | High |
| missing Desktop/iPhone render-only proof | no direct UI | future UI authority risk | Build render-only proof when UI exists. | Medium |
| missing Adapter transport-only proof | no route, fixtures only | future route wrong-owner risk | Add transport-only route proof if route added. | Medium |
| missing JD live acceptance | no live acceptance evidence | product readiness unknown | Add JD live Position acceptance pack. | High |

## 27. Recommended Future Build Slices

1. PH1.POSITION Repo-Truth Activation Pack.
2. Position Contract / State Machine Normalization.
3. Position Type Matrix.
4. Position Hierarchy / Reporting Line.
5. Position Requirement Schema Registry.
6. Dynamic Field / Requirement Overlay Map.
7. Country / Region / Industry / Company-Size Overlay Proof.
8. Position-To-Onboarding Requirement Handoff.
9. Position-To-Access Template Mapping.
10. Position Creation + Access Template Co-Authoring.
11. Position-To-Payroll/HR Boundary Map.
12. Position-To-Scheduler/Roster Boundary Map.
13. PH1.D + PH1.N Position Proposal Shell.
14. PH1.X Position Route/Risk Validation.
15. PH1.WRITE Position Guidance Boundary.
16. Position Audit Evidence Pack.
17. Position SQL Persistence / Migration Proof For Position Current + Lifecycle.
18. Workspace Scope Integration.
19. Promotion / Demotion / Position Change Flow.
20. Position Retirement Downstream Handoff To Access/ONB/Scheduler/HR.
21. Desktop/iPhone Render-Only Position Proof.
22. Adapter Transport-Only Position Proof.
23. JD Live Position Acceptance Pack.

## 28. What Codex Must Not Do

- do not invent position behavior.
- do not create duplicate position engine.
- do not let Access own position truth.
- do not let Position grant access directly.
- do not let Onboarding invent positions.
- do not let Payroll/HR be owned by Position.
- do not let Scheduler/Roster be owned by Position.
- do not let GPT-5.5/OpenAI create active positions directly.
- do not let Desktop/iPhone decide position/access truth.
- do not let Adapter decide position/access truth.
- do not use role strings as production position truth without canonical owner proof.
- do not treat `permission_profile_ref` as an access grant.
- do not treat `compensation_band_ref` as salary/payroll truth.
- do not treat `PositionScheduleType` as roster/scheduler truth.
- do not treat DocRequired as raw document ownership by PH1.POSITION.
- do not mutate existing onboarding sessions from schema change without explicit apply scope and backfill owner.
- do not delete old paths before proof.
- do not implement from this extraction document alone.

## 29. Final Extracted Architecture Sentence

Selene PH1.POSITION is the governed job/position truth boundary: it may define position titles, job families, departments, reporting lines, requirement schemas, and position-linked setup context where repo truth supports it, but Access must own permissions, Onboarding must own human setup progression, Payroll/HR must own employment and pay truth, Scheduler/Roster must own work allocation, and PH1.D/PH1.N/PH1.WRITE may only assist with understanding and presentation.
