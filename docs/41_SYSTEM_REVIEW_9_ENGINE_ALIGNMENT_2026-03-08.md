# Selene Full System Review - 9-Engine Alignment

Date: 2026-03-08
Scope: System_Core + Build Sections 01-09 + Build Execution Order vs current docs/code runtime.

## Input Verification
- Inserted and present:
  - `docs/CORE_ARCHITECTURE.md`
  - `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_01.md` ... `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_09.md`
  - `docs/SELENE_BUILD_EXECUTION_ORDER.md`
- Missing input artifact:
  - `Selene Authoritative Engine Inventory` standalone doc is not present under `docs/`.

## Readiness Snapshot (CURRENT -> TARGET)
- Section 01 Core Runtime Skeleton: IMPLEMENTED -> VERIFIED/GOVERNED pending governance-layer integration.
- Section 02 Session Engine: IMPLEMENTED -> cross-device continuity not complete.
- Section 03 Ingress + Turn: PARTIAL -> canonical envelope/response contract gaps.
- Section 04 Authority Layer: PARTIAL -> broad checks exist, but strict simulation-cert/policy envelope log discipline is incomplete.
- Section 05 Persistence + Sync: PARTIAL -> durable journal exists; full outbox/reconciliation contract missing.
- Section 06 Memory Engine: PARTIAL -> ledger-first core exists; full consistency/trust/graph governance surface not fully aligned.
- Section 07 Identity + Voice: PARTIAL -> enrollment/verification present; full trust-tier/cluster-drift/governance envelope coverage incomplete.
- Section 08 Platform Runtime: PARTIAL -> trigger gating exists; tablet class + platform identity/capability governance incomplete.
- Section 09 Runtime Governance Layer: NOT IMPLEMENTED as cross-runtime enforcement layer.

## Findings (Required Classification)

### F-01 Canonical doc hierarchy not yet switched to System_Core law
- CURRENT: `docs/00_INDEX.md`, `docs/05_OS_CONSTITUTION.md`, `docs/06_ENGINE_MAP.md`, `docs/07_ENGINE_REGISTRY.md` still anchor Option-B/legacy canon.
- TARGET: System_Core + Build Sections 01-09 + Build Execution Order are top-level canon.
- GAP: Documentation control plane points to prior architecture.
- CONFLICT: Active docs still declare old canon as authoritative.
- FIX REQUIRED: Re-point canonical index and governance docs to new law stack.
- MERGE REQUIRED: Merge old pointer docs into explicit legacy/archive status.
- RETIRE REQUIRED: Retire old canonical claims from Option-B docs.
- RENAME REQUIRED: Consider renaming `05_OS_CONSTITUTION.md` to legacy/pointer variant.
- OWNERSHIP GAP: No single doc owner currently enforcing canonical source-of-truth switch.
- DOC/CODE DRIFT: Yes.
- Evidence: `docs/00_INDEX.md:4-24`, `docs/05_OS_CONSTITUTION.md:76-149`, `docs/06_ENGINE_MAP.md:13-20`, `docs/07_ENGINE_REGISTRY.md:59-148`.

### F-02 Required standalone `Selene Authoritative Engine Inventory` doc missing
- CURRENT: No such standalone file in `docs/`.
- TARGET: Dedicated authoritative inventory doc per insertion law.
- GAP: One required architecture artifact absent.
- CONFLICT: Full-system review trigger prerequisites are incomplete by original workflow law.
- FIX REQUIRED: Add the missing doc verbatim once provided by JD.
- MERGE REQUIRED: N/A.
- RETIRE REQUIRED: N/A.
- RENAME REQUIRED: N/A.
- OWNERSHIP GAP: Architecture-doc ingestion pipeline lacks completeness check for mandatory set.
- DOC/CODE DRIFT: Docs-only gap.
- Evidence: `docs/` file inventory; no matching file name.

### F-03 Runtime still models platforms as IOS/ANDROID/DESKTOP only
- CURRENT: `AppPlatform` enum and parsing exclude `Tablet`.
- TARGET: Platform inventory includes iPhone, Android, Tablet, Desktop.
- GAP: Tablet platform class is not represented in runtime contracts.
- CONFLICT: System_Core declares Tablet as target class with trigger policy.
- FIX REQUIRED: Add `Tablet` across contract enum, parser, gating, and tests.
- MERGE REQUIRED: Merge Tablet behavior into PH1.OS capability and trigger policies.
- RETIRE REQUIRED: N/A.
- RENAME REQUIRED: N/A.
- OWNERSHIP GAP: PH1.OS ownership for tablet expansion is not codified in tracker status.
- DOC/CODE DRIFT: Yes.
- Evidence: `crates/selene_kernel_contracts/src/ph1link.rs:60-73`, `crates/selene_adapter/src/lib.rs:4038-4046`, `crates/selene_os/src/app_ingress.rs:3705-3710`.

### F-04 `/v1/voice/turn` response envelope lacks canonical session anchors
- CURRENT: Response returns status/outcome/reason/next_move/response_text/reason_code/provenance only.
- TARGET: Include canonical `session_id`, `turn_id`, `session_state`, execution metadata, sync state.
- GAP: Client cannot deterministically reconcile against authoritative session payload contract.
- CONFLICT: Violates System_Core session payload contract and Build Section 03 completion criteria.
- FIX REQUIRED: Extend voice response schema and populate from authoritative runtime outcome.
- MERGE REQUIRED: Merge session snapshot into adapter response mapping path.
- RETIRE REQUIRED: Legacy response-only shape should be retired.
- RENAME REQUIRED: N/A.
- OWNERSHIP GAP: Adapter/API contract owner not yet bound to new System_Core payload law.
- DOC/CODE DRIFT: Yes.
- Evidence: `crates/selene_adapter/src/lib.rs:264-272`, `crates/selene_adapter/src/lib.rs:4576-4593`, `docs/CORE_ARCHITECTURE.md` (Universal session payload contract section).

### F-05 Session continuity still scoped by actor + device, not shared cloud session
- CURRENT: Session lookup filters by `user_id && device_id` and reuses latest per-device session.
- TARGET: Cross-device attach to canonical cloud session with deterministic ordering.
- GAP: True multi-device session continuity and convergence is incomplete.
- CONFLICT: Violates System_Core cross-device session attachment law.
- FIX REQUIRED: Introduce canonical actor/session resolver independent of device identity.
- MERGE REQUIRED: Merge per-device timelines under canonical session ledger rules.
- RETIRE REQUIRED: Retire device-bound session lookup as primary resolver.
- RENAME REQUIRED: Consider renaming helper to reflect new semantics (`latest_session_for_actor`).
- OWNERSHIP GAP: PH1.L cross-device continuation milestone not closed.
- DOC/CODE DRIFT: Yes.
- Evidence: `crates/selene_adapter/src/lib.rs:5359-5384`, `crates/selene_adapter/src/lib.rs:5476-5486`.

### F-06 Ingress execution envelope is partial; request_id/trace/admission state not canonicalized end-to-end
- CURRENT: Header checks happen in HTTP adapter; core ingress request struct does not carry canonical request_id/trace_id/admission envelope fields.
- TARGET: Runtime Execution Envelope includes request_id, trace_id, idempotency_key, platform/device context, admission state, session/turn placeholders and is propagated downstream.
- GAP: Boundary metadata is split between transport guard and internal runtime objects.
- CONFLICT: Build Section 03 envelope discipline is incomplete.
- FIX REQUIRED: Define and enforce explicit `RuntimeExecutionEnvelope` struct at ingress boundary; pass through all gates.
- MERGE REQUIRED: Merge transport security metadata into canonical runtime envelope.
- RETIRE REQUIRED: Retire ad-hoc metadata generation paths.
- RENAME REQUIRED: N/A.
- OWNERSHIP GAP: No single envelope type owner across adapter + ingress runtime.
- DOC/CODE DRIFT: Yes.
- Evidence: `crates/selene_adapter/src/bin/http_adapter.rs:511-547`, `crates/selene_os/src/app_ingress.rs:106-117`, `crates/selene_os/src/app_ingress.rs:3070-3113`.

### F-07 Failure contract uses raw reasons/status mapping, not canonical deterministic failure classes
- CURRENT: Rejections map to HTTP status + free-form reason strings.
- TARGET: Canonical failure classes (authn/authz/invalid_payload/replay/session_conflict/policy_violation/execution_failure/retryable).
- GAP: Client handling semantics can drift by reason string.
- CONFLICT: Section 03 deterministic error contract not fully implemented.
- FIX REQUIRED: Add stable failure_class enum in response envelope and map all gate failures deterministically.
- MERGE REQUIRED: Merge current reason strings into structured class + reason_code model.
- RETIRE REQUIRED: Retire reason-only client branching.
- RENAME REQUIRED: N/A.
- OWNERSHIP GAP: Error taxonomy owner not defined.
- DOC/CODE DRIFT: Yes.
- Evidence: `crates/selene_adapter/src/bin/http_adapter.rs:943-1012`.

### F-08 Persistence + Sync has journal replay but not full durable outbox/reconciliation contract
- CURRENT: Adapter persists JSONL journal of voice requests and replays it; queue counters exist.
- TARGET: Durable outbox with ack state, retry policy engine, conflict severity classes, recovery modes, cross-device/cross-node dedupe consensus.
- GAP: Recovery posture and reconciliation policy are not modeled as canonical contracts.
- CONFLICT: Build Section 05 completion criteria only partially satisfied.
- FIX REQUIRED: Introduce first-class outbox records + reconciliation state machine + recovery modes.
- MERGE REQUIRED: Merge journal + sync worker + store indexes into one persistence backbone module.
- RETIRE REQUIRED: Retire journal-only persistence as primary recovery model.
- RENAME REQUIRED: N/A.
- OWNERSHIP GAP: PH1.F/PH1.L/adapter shared ownership boundary unclear for outbox authority.
- DOC/CODE DRIFT: Yes.
- Evidence: `crates/selene_adapter/src/lib.rs:589-599`, `crates/selene_adapter/src/lib.rs:3903-3943`, `crates/selene_adapter/src/lib.rs:3947-3975`.

### F-09 Section 09 Runtime Governance Layer is not present as cross-runtime law-enforcement subsystem
- CURRENT: `PH1.GOV` exists but governs artifact/rollout decisions, not runtime-wide invariant enforcement/safe-mode/quarantine/cluster governance.
- TARGET: Dedicated Runtime Governance Layer enforcing architecture invariants across Sections 01-08.
- GAP: No dedicated governance rule registry/policy version/severity-response runtime for law enforcement.
- CONFLICT: Build Section 09 completion criteria unmet.
- FIX REQUIRED: Implement `runtime_governance` subsystem with decision log, policy versioning, rule registry, severity/response model, quarantine + safe mode.
- MERGE REQUIRED: Merge existing PH1.GOV signals into Section09 layer as one input source.
- RETIRE REQUIRED: Retire assumption that PH1.GOV == runtime governance.
- RENAME REQUIRED: N/A.
- OWNERSHIP GAP: Governance ownership currently split between PH1.GOV and PH1.OS without Section09 owner.
- DOC/CODE DRIFT: Yes.
- Evidence: `crates/selene_os/src/ph1gov.rs:27-44`; no separate runtime governance module file under `crates/selene_os/src/`.

### F-10 Engine registry marks PH1.M as Non-Authoritative, conflicting with new memory-authority law
- CURRENT: Registry row labels PH1.M as Non-Authoritative.
- TARGET: Memory engine is cloud-authoritative for persistent knowledge.
- GAP: Engine authority classification is stale.
- CONFLICT: Violates Section 06 authoritative knowledge boundary.
- FIX REQUIRED: Update authority classification and ownership language across registry/matrices.
- MERGE REQUIRED: Merge legacy memory wording into new ledger-authority semantics.
- RETIRE REQUIRED: Retire old "non-authoritative memory" classification.
- RENAME REQUIRED: N/A.
- OWNERSHIP GAP: Memory ownership semantics not harmonized across docs.
- DOC/CODE DRIFT: Yes.
- Evidence: `docs/07_ENGINE_REGISTRY.md:93`.

### F-11 Tracker and coverage docs overstate completion vs newly declared architecture gaps
- CURRENT: Many rows are `DONE` with "none" blockers including PH1.OS/PH1.VOICE.ID/PH1.M.
- TARGET: Track open P0/P1/P2 gaps from System_Core until code+tests+docs close them.
- GAP: Governance/reporting artifacts do not reflect new gap baseline.
- CONFLICT: Operational planning can falsely signal readiness.
- FIX REQUIRED: Re-baseline tracker/coverage/closure plan against System_Core Known Architectural Gaps.
- MERGE REQUIRED: Merge previous closure tracker with new gap taxonomy.
- RETIRE REQUIRED: Retire outdated DONE assertions for unresolved areas.
- RENAME REQUIRED: N/A.
- OWNERSHIP GAP: Program management owner for architecture drift closure unclear.
- DOC/CODE DRIFT: Yes.
- Evidence: `docs/COVERAGE_MATRIX.md:13,54,63`, `docs/33_ENGINE_REVIEW_TRACKER.md:131-151`, `docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md:1049`.

## Integration Plan: 9 Engines With Existing Engine Landscape
- Keep existing specialized engines as subordinate capability engines, not constitutional peers.
- Bind all specialized engines behind the 9-engine control path:
  1. Section01/02 own runtime + session container.
  2. Section03 owns all ingress normalization/envelope and canonical API boundary.
  3. Section04 owns authority decisions before any protected action.
  4. Section05 owns persistence/reconcile/dedupe correctness.
  5. Section06 owns memory truth and retrieval governance.
  6. Section07 owns biometric identity gate and artifact lifecycle.
  7. Section08 owns platform normalization/capability/trust posture.
  8. Section09 owns invariant enforcement and fail-safe governance.
- Existing engines like PH1.NLP/PH1.E/PH1.BCAST/PH1.DELIVERY/PH1.CAPREQ remain execution participants but must route through these 9 control layers.

## Step-by-Step Remediation Plan (Sweeps)

### Sweep 1 - Canonical Documentation Reset
- Update canonical index and law pointers to System_Core + Sections 01-09 + Build Execution Order.
- Add missing `Selene Authoritative Engine Inventory` doc.
- Re-baseline tracker and coverage docs to new gap model.

### Sweep 2 - Ingress Envelope + Response Contract Hardening (Section03)
- Introduce explicit runtime execution envelope type with required fields.
- Thread envelope across adapter -> app ingress -> execution gates.
- Expand `/v1/voice/turn` response to include `session_id`, `turn_id`, `session_state`, metadata and failure_class.

### Sweep 3 - Session Convergence and Cross-Device Continuity (Section02 + Section05)
- Replace actor+device lookup with canonical actor/session resolver.
- Add deterministic session attach/resume semantics for second device.
- Enforce canonical turn ordering and stale-operation rejection across devices.

### Sweep 4 - Platform Runtime Completion (Section08)
- Add `Tablet` platform to contracts, parser, trigger policy, tests.
- Introduce platform identity model fields + capability registry + negotiation result object.
- Add client integrity verification outputs and device trust posture in envelope.

### Sweep 5 - Persistence + Sync Backbone Upgrade (Section05)
- Implement durable outbox schema and integrity checks.
- Implement reconciliation policy engine with recovery modes and conflict severity classes.
- Add explicit cross-node dedupe consensus state and audit entries.

### Sweep 6 - Runtime Governance Layer Build-Out (Section09)
- Implement governance rule registry, policy version, severity/response classes.
- Add runtime quarantine/safe-mode flows and cross-node governance consistency checks.
- Add governance decision log + replay audit trail.

### Sweep 7 - Authority/Memory/Identity Final Harmonization (Sections04/06/07)
- Align PH1.M authority labels and docs to cloud-authoritative memory law.
- Expand identity trust-tier naming/consistency-level exposure to Section07 contract vocabulary.
- Tighten simulation-certification and onboarding-readiness gates as explicit envelope outcomes.

## Estimated Sweep Count
- Required sweeps for full alignment: **7**.

## Files That Must Change (Initial Set)
- Docs control plane:
  - `docs/00_INDEX.md`
  - `docs/05_OS_CONSTITUTION.md`
  - `docs/06_ENGINE_MAP.md`
  - `docs/07_ENGINE_REGISTRY.md`
  - `docs/COVERAGE_MATRIX.md`
  - `docs/33_ENGINE_REVIEW_TRACKER.md`
  - `docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md`
  - `docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md` (new)
- Ingress/API/runtime boundary:
  - `crates/selene_adapter/src/bin/http_adapter.rs`
  - `crates/selene_adapter/src/lib.rs`
  - `crates/selene_os/src/app_ingress.rs`
  - `crates/selene_kernel_contracts/src/ph1link.rs` (AppPlatform)
- Governance/persistence expansion:
  - `crates/selene_os/src/ph1gov.rs` (integration input only)
  - `crates/selene_os/src/` (new Section09 runtime governance module files)
  - `crates/selene_storage/src/ph1f.rs` (outbox/reconciliation/recovery governance storage)

## Engine Update/Merge/Retire Decisions
- UPDATE REQUIRED:
  - Sections 01-09 integration boundaries in adapter + app_ingress + contracts.
  - PH1.L, PH1.OS, PH1.M, PH1.VOICE.ID, PH1.GOV, PH1.F.
- MERGE REQUIRED:
  - Merge current PH1.GOV artifact-governance signals into new Section09 runtime governance layer.
  - Merge journal + sync-worker persistence pieces into a formal Section05 persistence backbone.
- RETIRE REQUIRED:
  - Retire legacy "Option-B canonical" status from old control docs.
  - Retire PH1.M non-authoritative labeling.
- RENAME REQUIRED:
  - Optional but recommended: rename legacy canonical docs to explicit `*_LEGACY_*` where still needed.

## Whether Any of the 9 New Engines Need Adjustment
- Yes.
  - Section08 needs explicit Tablet-first contract inclusion to match System_Core platform inventory.
  - Section09 needs an implementation profile that explicitly maps existing PH1.GOV/PH1.OS signals into governance-layer rule inputs to avoid duplicate policy engines.
  - Section03 should explicitly define a typed failure-class enum and canonical response schema as contract artifacts (not adapter-local conventions).

## Safe-Operation Posture Until Remediation Completes
- Keep fail-closed behavior as default for ingress security and identity-dependent actions.
- Treat docs `COVERAGE_MATRIX` / tracker DONE statuses as non-authoritative until Sweep 1 re-baseline completes.
- Block new feature expansion that depends on cross-device continuity, tablet runtime parity, or runtime governance safe-mode until corresponding sweeps close.
