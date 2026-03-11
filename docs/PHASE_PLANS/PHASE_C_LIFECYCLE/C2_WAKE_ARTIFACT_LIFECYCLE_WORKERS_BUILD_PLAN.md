PHASE C2 — WAKE ARTIFACT LIFECYCLE WORKERS BUILD PLAN

A) REPO TRUTH CHECK
- `git status --short`: empty
- current branch: `main`
- repo-truth snapshot HEAD at review start: `31501f47918c299947396a36ce5cac40e7d5af31`
- snapshot semantics: this value records the repo state reviewed at the start of this pass; it is not required to equal the later commit hash that records this document update.
- target file at review start: present; tracked C2 hardening target
- exact files reviewed:
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_C_LIFECYCLE/C1_LIFECYCLE_MODEL_REVIEW.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_C_LIFECYCLE/C1_LIFECYCLE_MODEL_REVIEW.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_C_LIFECYCLE/C2_WAKE_ARTIFACT_LIFECYCLE_WORKERS_BUILD_PLAN.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_C_LIFECYCLE/C2_WAKE_ARTIFACT_LIFECYCLE_WORKERS_BUILD_PLAN.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/WAKE_BUILD_PLAN.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/WAKE_BUILD_PLAN.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/CORE_ARCHITECTURE.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/CORE_ARCHITECTURE.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/SELENE_BUILD_EXECUTION_ORDER.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/SELENE_BUILD_EXECUTION_ORDER.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_01.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_01.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_03.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_03.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_05.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_05.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_08.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_08.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_09.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_09.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_11.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_11.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/ARTIFACTS_LEDGER_TABLES.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/ARTIFACTS_LEDGER_TABLES.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_J.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_J.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_GOV.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_GOV.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_LAW.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_LAW.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_OS.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_OS.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1art.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1art.rs)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_storage/src/ph1f.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_storage/src/ph1f.rs)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_storage/src/ph1j.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_storage/src/ph1j.rs)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/device_artifact_sync.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/device_artifact_sync.rs)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_B_ANDROID_WAKE/B3_ANDROID_RUNTIME_ENFORCEMENT_WIRING_BUILD_PLAN.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_B_ANDROID_WAKE/B3_ANDROID_RUNTIME_ENFORCEMENT_WIRING_BUILD_PLAN.md)
- [/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_B_ANDROID_WAKE/B4_PARITY_TESTS_FAILURE_HANDLING_BUILD_PLAN.md](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_B_ANDROID_WAKE/B4_PARITY_TESTS_FAILURE_HANDLING_BUILD_PLAN.md)

B) PURPOSE
- C2 defines the worker design for wake artifact lifecycle handling only.
- C2 consumes the frozen C1 lifecycle model and current wake repo truth to define:
- which wake workers exist
- which lifecycle plane each worker may write
- which rows are authoritative
- which surfaces are receipts, projections, or decision-only inputs
- how retries, replay, collision handling, and compensation must behave
- C2 is planning only.
- C2 does not implement code, tests, storage rewrites, memory lifecycle enforcement, Android parity work, or cross-subsystem proof/law redesign.

C) DEPENDENCY RULE
- C2 depends on the frozen C1 lifecycle model and must consume it without redefining:
- lifecycle state names
- lifecycle planes
- subject identity rules
- receipt-vs-authority law
- collision law
- proof / governance / law ownership boundaries
- C2 also depends on:
- Section 01 for authoritative runtime boundary
- Section 03 for canonical ingress and execution-envelope discipline
- Section 05 for idempotency, retry, reconciliation, and durable outbox behavior
- Section 08 for platform/runtime normalization boundaries
- Section 09 for governance enforcement posture
- Section 11 for final runtime law posture
- Phase A remains frozen.
- Phase B remains approved and must not be silently redefined.
- Safest planning assumption when repo truth is incomplete:
- use the current authoritative surface already present in repo
- record the missing normalization explicitly
- do not invent a new lifecycle path to hide the gap

D) ARCHITECTURAL POSITION
- Selene remains cloud-authoritative.
- Client devices never become lifecycle authority.
- C2 sits between already-published wake artifacts and device-local wake artifact application.
- C2 therefore spans three canonical C1 planes:
- `ARTIFACT_IDENTITY_LIFECYCLE`
- `TARGET_DEPLOYMENT_LIFECYCLE`
- read-only `RUNTIME_USE_LIFECYCLE`
- C2 does not own `MEMORY_LIFECYCLE`.
- Current wake repo truth already splits across:
- append-only artifact identity storage in `os_process.artifacts_ledger`
- rollout projection storage in `wake_promotion_ledger` and `wake_promotion_current`
- per-device deployment storage in `wake_artifact_apply_ledger` and `wake_artifact_apply_current`
- runtime-use evidence in `WakeRuntimeEventRecord`
- audit/proof visibility in PH1.J
- governance and law decision posture in PH1.GOV and PH1.LAW
- C2 exists to stop those surfaces from being treated as one flat state machine.

E) C1 LIFECYCLE ASSUMPTIONS CONSUMED
- C2 consumes the frozen C1 subject identity law:
- artifact identity subject:
- `(ARTIFACT_IDENTITY_LIFECYCLE, ARTIFACT_IDENTITY_SUBJECT, artifact_identity_ref or artifact_id, governed artifact scope, lineage_root_or_parent_ref)`
- target deployment subject:
- `(TARGET_DEPLOYMENT_LIFECYCLE, TARGET_DEPLOYMENT_INSTANCE, device_id + artifact_version or equivalent deployment subject, DEVICE_LOCAL_SCOPE or TARGET_DEPLOYMENT_SCOPE, parent artifact identity ref)`
- runtime-use subject:
- `(RUNTIME_USE_LIFECYCLE, RUNTIME_CONSUMPTION_INSTANCE, wake_event_id or equivalent runtime-use record id, SESSION_SCOPE or DEVICE_LOCAL_SCOPE, parent active artifact ref)`
- C2 consumes the frozen C1 rules that:
- artifact-global truth must not be overwritten by target-local receipts
- target deployment truth must not be inferred from runtime-use evidence
- runtime-use evidence must not be mistaken for activation truth
- replacement and rollback must preserve lineage and historical visibility
- duplicate apply or promotion attempts must be idempotent and replay-safe
- invalid transitions must fail closed
- PH1.J is visibility only, not lifecycle authority
- PH1.GOV and PH1.LAW are decision-only consumers, not storage writers
- Memory lifecycle is explicitly out of scope for C2 except:
- read-only awareness that memory remains a separate owned lifecycle plane
- downstream boundary notes that defer memory retention / archive / delete / restore work to C3
- Repo-truth ambiguity that C2 must carry explicitly:
- `WakePromotionState::{Candidate, Shadow, Canary, Active, Blocked, RolledBack}` is a wake rollout projection, not the canonical C1 state family.
- `WakeArtifactApplyState::{Staged, Active, RolledBack}` is a current deployment projection, not a full stored materialization of every canonical C1 target-deployment state.
- `WakeRuntimeEventRecord` is runtime-use evidence, not a canonical activation table.

F) CURRENT REPO SURFACES IN SCOPE
Current Repo Surface → C2 Worker Scope Mapping
| repo surface | lifecycle plane | subject class | current role | C2 worker relevance | authoritative or receipt-only | notes / constraints |
| --- | --- | --- | --- | --- | --- | --- |
| `os_process.artifacts_ledger` | `ARTIFACT_IDENTITY_LIFECYCLE` | `ARTIFACT_IDENTITY_SUBJECT` | append-only artifact-global row by scope/type/version | identity-plane source of truth; C2 must read it and use it for any artifact-global wake transition write | authoritative | one row per `(scope_type, scope_id, artifact_type, artifact_version)`; rollback/deprecation is append-only, never overwrite |
| `wake_promotion_ledger` | rollout-scoped bridge between artifact identity and target deployment | `WAKE_ROLLOUT_PROJECTION` | promotion history for `Candidate / Shadow / Canary / Active / Blocked / RolledBack` | owned by promotion worker family | authoritative projection | projection only; must not replace artifact-global truth in `artifacts_ledger` |
| `wake_promotion_current` | rollout-scoped bridge between artifact identity and target deployment | `WAKE_ROLLOUT_PROJECTION` | current rollout state and active-pointer slice | owned by promotion worker family | authoritative projection | `Active` here is not sufficient by itself to redefine artifact-global lifecycle law |
| `wake_artifact_apply_ledger` | `TARGET_DEPLOYMENT_LIFECYCLE` | `TARGET_DEPLOYMENT_INSTANCE` | per-device stage / activate / rollback history | owned by target deployment worker family | authoritative | idempotent by `(device_id, artifact_version, state, idempotency_key)` |
| `wake_artifact_apply_current` | `TARGET_DEPLOYMENT_LIFECYCLE` | `TARGET_DEPLOYMENT_INSTANCE` | per-device staged version, active version, last-known-good pointer, rollback reason | owned by target deployment worker family | authoritative | last-known-good pointer is the canonical C2 compensation anchor for device rollback |
| `WakeRuntimeEventRecord` / `wake_runtime_events` | `RUNTIME_USE_LIFECYCLE` | `RUNTIME_CONSUMPTION_INSTANCE` | accepted/rejected wake runtime evidence with model version, thresholds, and window timing | read-only runtime-use evidence input for C2; never worker-owned write target | authoritative runtime-use evidence only | there is no separate runtime-use current table in the current slice; C2 must not promote this to activation truth |
| `device_artifact_sync_*` queue surfaces in `ph1f.rs` | operational receipt/evidence support surface | `SYNC_JOB` | dequeue / ack / fail / dead-letter and replay handling for artifact sync work | owned by receipt-delivery worker family | receipt-only for lifecycle semantics | queue state is operational control flow only, not artifact identity or target deployment truth |
| `audit.audit_events` | evidence overlay across lifecycle planes | `LIFECYCLE_EVENT_EVIDENCE` | reason-coded `STATE_TRANSITION` and related audit rows | all C2 worker families must emit reason-coded visibility when lifecycle-significant | authoritative for audit only | PH1.J must never become a parallel lifecycle state store |
| `RuntimeExecutionEnvelope.artifact_trust_state` plus canonical proof refs | guarded decision input, not a lifecycle row | `ARTIFACT_TRUST_EXECUTION_STATE` | Phase A trust/proof state carried through the runtime envelope | required read input before protected wake activation or replacement transitions | authoritative decision input only | C2 may consume it; C2 may not mint new trust truth |
| `gov_decision_bundle` and governance execution state | governed decision surface | `GOVERNANCE_DECISION` | allow/block posture for activation / deprecation / rollback when enterprise support is enabled | required read input for governed promotion and identity transitions | authoritative decision-only | PH1.GOV owns the decision, not the storage commit |
| `RuntimeExecutionEnvelope.law_state` | final protected-execution law surface | `RUNTIME_LAW_DECISION` | final runtime law posture for protected completion | required read input when wake transition is protected-execution relevant | authoritative decision-only | PH1.LAW owns the decision, not the lifecycle write |

G) CANONICAL LIFECYCLE PLANES IN C2 SCOPE
- `ARTIFACT_IDENTITY_LIFECYCLE`
  - in scope for C2 authoritative writes
  - C2 may only write wake artifact identity truth through append-only `artifacts_ledger`
  - C2 states materially in scope:
  - `PUBLISHED` as a precondition only
  - `ACTIVE`
  - `REPLACED`
  - `ROLLED_BACK`
  - `REVOKED`, `EXPIRED`, `ARCHIVED`, `PURGED`, and `RESTORED` are not C2-authored state transitions in the current wake slice
- `TARGET_DEPLOYMENT_LIFECYCLE`
  - in scope for C2 authoritative writes
  - current repo projection maps:
  - `WakeArtifactApplyState::Staged` to current wake install/apply-readiness posture
  - `WakeArtifactApplyState::Active` to target-local active deployment posture
  - `WakeArtifactApplyState::RolledBack` to target-local rollback posture
  - current repo does not store a standalone authoritative `APPLY_REQUESTED` row for wake deployment
  - safest C2 planning assumption:
  - request/pull/apply intent may exist in worker control flow and receipts
  - authoritative stored wake deployment truth remains `Staged`, `Active`, and `RolledBack` until later lifecycle storage normalization
- `RUNTIME_USE_LIFECYCLE`
  - in scope for read-only consumption only
  - C2 must explicitly name runtime-use so it is not blurred into activation truth
  - current repo truth is `WakeRuntimeEventRecord`
  - C2 workers do not author runtime-use state
  - C2 workers may only observe runtime-use evidence for rollback posture, visibility, and downstream law integration
- `MEMORY_LIFECYCLE`
  - explicitly out of scope for C2
  - no C2 worker may write memory retention / archive / purge / restore posture
  - any memory-lifecycle downstream consequence is deferred to C3

H) WORKER FAMILIES
- `WakeArtifactIdentityTransitionWorker`
  - owns canonical wake artifact identity transitions that must land in `artifacts_ledger`
  - consumes promotion outcomes and governed allow/block posture
  - never writes per-device deployment state
- `WakePromotionProjectionWorker`
  - owns the wake rollout projection state machine in `wake_promotion_ledger` and `wake_promotion_current`
  - maps projection states to C1 lifecycle meaning without replacing artifact identity truth
  - owns blocked-version revalidation and rollback projection handling
- `WakeTargetDeploymentWorker`
  - owns per-device wake pull, stage, activate, and rollback work
  - writes only `wake_artifact_apply_ledger` and `wake_artifact_apply_current`
  - uses last-known-good recovery and fail-closed transition validation
- `WakeSyncReceiptDeliveryWorker`
  - owns sync-envelope dequeue, ACK, retry, replay-due, and dead-letter handling
  - does not own lifecycle truth
  - exists so receipt delivery cannot be mistaken for activation or artifact identity state

Worker Family Matrix
| worker family | lifecycle plane(s) touched | authoritative write target | receipt/evidence emissions | trigger source | idempotency key / ordering basis | forbidden writes |
| --- | --- | --- | --- | --- | --- | --- |
| `WakeArtifactIdentityTransitionWorker` | `ARTIFACT_IDENTITY_LIFECYCLE` | append-only `os_process.artifacts_ledger` rows for wake artifact identity state changes | PH1.J `STATE_TRANSITION` audit; governed trust/proof refs already carried in the envelope | promotion projection reaches a governed commit point such as activation or explicit rollback finalization | `artifacts_ledger` idempotency scope `(scope_type, scope_id, artifact_type, artifact_version, idempotency_key)` plus append order by `artifact_id` | `wake_promotion_current`, `wake_artifact_apply_current`, `wake_runtime_events`, direct PH1.GOV/PH1.LAW writes |
| `WakePromotionProjectionWorker` | rollout-scoped bridge across artifact identity and target deployment | `wake_promotion_ledger`, `wake_promotion_current`, active-pointer slice | PH1.J `STATE_TRANSITION` audit and operational rollout receipts only | governed rollout request, blocked-version revalidation, explicit rollback direction | `(artifact_version, to_state, idempotency_key)` plus deterministic current-state validation; blocked versions require explicit revalidation | `artifacts_ledger` overwrite or in-place mutation, `wake_artifact_apply_*`, `wake_runtime_events`, direct PH1.GOV/PH1.LAW writes |
| `WakeTargetDeploymentWorker` | `TARGET_DEPLOYMENT_LIFECYCLE`; read-only `ARTIFACT_IDENTITY_LIFECYCLE` and `RUNTIME_USE_LIFECYCLE` inputs | `wake_artifact_apply_ledger`, `wake_artifact_apply_current`, blocked-version projection for a device/version | operational sync receipts, PH1.J audit when lifecycle-significant, local cache ref only as assist metadata | device pull/apply pass, pull response updates, activation hook failure, explicit rollback posture | `(device_id, artifact_version, state, idempotency_key)` plus per-device current row and `apply_event_id` ordering | `artifacts_ledger`, `wake_promotion_*`, `wake_runtime_events`, direct PH1.GOV/PH1.LAW writes |
| `WakeSyncReceiptDeliveryWorker` | no authoritative lifecycle plane writes; receipt-only support for artifact identity and target deployment | queue ACK / fail / dead-letter state only through `device_artifact_sync_*` queue surfaces | external sync envelopes and bounded send error evidence | queued sync jobs, replay-due rows, sender ACK/NACK posture | sync job lease order plus bounded attempt count; queue replay uses prior sync job identity | `artifacts_ledger`, `wake_promotion_*`, `wake_artifact_apply_*`, `wake_runtime_events`, any artifact-trust or governance state |

I) WAKE ARTIFACT LIFECYCLE TRANSITIONS IN SCOPE
- C2 in-scope transitions are only the wake worker transitions required to move from already-published wake artifact versions to governed rollout and per-device activation.
- Current repo truth that C2 must preserve explicitly:
- `WakePromotionState` controls rollout projection only.
- `WakeArtifactApplyState` controls per-device deployment only.
- `WakeRuntimeEventRecord` is runtime-use evidence only.
- Therefore C2 must map current wake projection states to canonical C1 lifecycle meaning without pretending current repo already persists every canonical state as a standalone row.

Transition Ownership Matrix
| transition | lifecycle plane | authoritative writer | receipt emitter | prerequisite state | success result | failure result | deferred integration dependency if any |
| --- | --- | --- | --- | --- | --- | --- | --- |
| wake artifact admitted into rollout candidate projection | rollout-scoped bridge | `WakePromotionProjectionWorker` | builder/release operational receipt where present | wake artifact already exists in `artifacts_ledger` as a published artifact candidate | `wake_promotion_current.state=Candidate` for that version | no projection write; request is refused fail-closed | artifact-global `PUBLISHED` truth remains upstream and is not reauthored by C2 |
| `Candidate -> Shadow` | rollout-scoped bridge | `WakePromotionProjectionWorker` | operational promotion receipt only | current projection state is `Candidate` | `wake_promotion_current.state=Shadow` and ledger row append | contract violation; no state change | none |
| `Shadow -> Canary` | rollout-scoped bridge | `WakePromotionProjectionWorker` | operational promotion receipt only | current projection state is `Shadow` | `wake_promotion_current.state=Canary` and ledger row append | contract violation; no state change | none |
| `Canary -> Active` | rollout-scoped bridge leading into artifact identity activation | `WakePromotionProjectionWorker` first, then `WakeArtifactIdentityTransitionWorker` for artifact-global truth | PH1.J audit required; governance/law visibility when enabled | current projection state is `Canary`; `active_gate_passed=true`; required governance/law posture is non-blocking | rollout projection becomes `Active`; artifact-global wake version becomes canonical `ACTIVE`; prior active version becomes `REPLACED` at artifact identity plane where applicable | contract violation or governed block; no activation commit | C4 later standardizes end-to-end lifecycle proof/governance/law linkage format, but not the worker boundary |
| `Active -> RolledBack` in promotion projection | rollout-scoped bridge with artifact identity rollback consequence | `WakePromotionProjectionWorker` first, then `WakeArtifactIdentityTransitionWorker` for artifact-global rollback history | PH1.J audit required; governance/law visibility when enabled | current projection state is `Active`; explicit rollback reason exists | projection becomes `RolledBack`; active pointer clears; artifact identity records `ROLLED_BACK` and prior lawful active target becomes the recovered subject where applicable | contract violation; no rollback write | C4 later standardizes global rollback evidence linkage across subsystems |
| pull/update receipt -> staged install projection | `TARGET_DEPLOYMENT_LIFECYCLE` | `WakeTargetDeploymentWorker` | operational pull/apply receipt only | device exists; artifact update exists; payload ref and package hash are syntactically valid | `wake_artifact_apply_ledger` append with `Staged`; current row sets `staged_artifact_version` | contract violation or immediate compensation path; no active-state overwrite | current repo does not store standalone `APPLY_REQUESTED`; C2 keeps that as control flow / receipt-only |
| `Staged -> target-local ACTIVE` | `TARGET_DEPLOYMENT_LIFECYCLE` | `WakeTargetDeploymentWorker` | local activation receipt only | staged row exists for the same device/version; staged pointer matches; artifact is not already active | `wake_artifact_apply_current.active_artifact_version=artifact_version`; `activated_at` set; `last_known_good` preserved; ledger append with `Active` | contract violation, no-op replay, or rollback compensation if later activation hook fails | C4 later normalizes canonical `APPLY_CONFIRMED` visibility if required beyond current projection |
| staged or active target-local deployment -> `ROLLED_BACK` | `TARGET_DEPLOYMENT_LIFECYCLE` | `WakeTargetDeploymentWorker` | rollback operational receipt only | source staged or active row exists for the same device/version | `wake_artifact_apply_current.active_artifact_version=last_known_good`; rollback reason recorded; ledger append with `RolledBack`; blocked version projection updated | contract violation; no state change | none |
| target-local `ACTIVE` -> runtime-use evidence | `RUNTIME_USE_LIFECYCLE` | existing PH1.W runtime event writer, not a C2 lifecycle worker | runtime event emission itself | device has active wake artifact and a runtime wake decision occurs | `WakeRuntimeEventRecord` append with accepted/rejected outcome, scores, thresholds, and model version | no runtime-use evidence row or explicit runtime reason-code refusal | C4 later standardizes runtime-use lifecycle visibility against the broader lifecycle schema |

J) AUTHORITATIVE WRITES VS RECEIPT EMISSIONS
- Authoritative writes in C2 are limited to:
- `os_process.artifacts_ledger` for artifact-global wake identity truth
- `wake_promotion_ledger` and `wake_promotion_current` for rollout projection truth
- `wake_artifact_apply_ledger` and `wake_artifact_apply_current` for target-local deployment truth
- `wake_runtime_events` for runtime-use evidence, but not by the C2 worker families defined here
- Receipt / evidence / projection surfaces in C2 are:
- pull responses and sync envelopes
- local cache paths and payload download results
- retry / ACK / dead-letter queue state
- PH1.J audit rows
- PH1.GOV decision bundles
- PH1.LAW law state
- PH1.OS posture normalization
- Hard law for C2:
- receipt presence never substitutes for artifact identity or target deployment truth
- queue ACK state never substitutes for lifecycle completion
- runtime-use evidence never substitutes for artifact activation truth
- governance or law decision bundles never substitute for storage commits

Authoritative Commit vs Receipt / Projection Split Law
- Each C2 worker must land the authoritative commit for its owned plane before it emits PH1.J evidence, operational receipts, or downstream recovery work for that same transition.
- `WakeArtifactIdentityTransitionWorker` commits append-only `artifacts_ledger` truth first; `wake_promotion_*`, PH1.J, sync envelopes, and runtime-use evidence may reflect that result later but may not pre-authorize it.
- `WakePromotionProjectionWorker` commits `wake_promotion_ledger` and `wake_promotion_current` first for rollout projection truth, but that projection remains rollout-scoped only and never upgrades itself into artifact-global identity authority.
- `WakeTargetDeploymentWorker` commits `wake_artifact_apply_ledger` and `wake_artifact_apply_current` first for target deployment truth; local apply success, client acknowledgement, cache writes, and runtime-use events are downstream evidence only.
- `WakeSyncReceiptDeliveryWorker` owns queue-control commits only. Queue `ACK`, retry, replay-due, and dead-letter outcomes are authoritative for queue handling, not for lifecycle planes.
- PH1.J, PH1.GOV, PH1.LAW, and PH1.OS remain visibility or decision consumers at C2 scope. They may gate or record transitions, but they do not become lifecycle writers.

Authoritative Commit vs Receipt / Projection Matrix
| repo surface | lifecycle plane | authority class | authoritative write or receipt/projection/evidence only | writer allowed | consumers allowed | must never be treated as | notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `os_process.artifacts_ledger` | `ARTIFACT_IDENTITY_LIFECYCLE` | artifact-global lifecycle authority | authoritative write | `WakeArtifactIdentityTransitionWorker` only | promotion worker, target deployment worker, PH1.J, PH1.GOV, PH1.LAW, PH1.OS, operators | rollout projection, device-local apply success, runtime-use evidence, queue ACK | append-only truth keyed by `(scope_type, scope_id, artifact_type, artifact_version)` with idempotency dedupe |
| `wake_promotion_ledger` | rollout-scoped bridge | rollout history authority only | authoritative projection | `WakePromotionProjectionWorker` only | identity worker, deployment worker, PH1.J, operators | artifact-global `ACTIVE` / `REPLACED` / `ROLLED_BACK` truth | authoritative only for rollout history in the wake bridge slice |
| `wake_promotion_current` | rollout-scoped bridge | rollout current-pointer authority only | projection only | `WakePromotionProjectionWorker` only | identity worker, deployment worker, operators | artifact identity source of truth | current pointer may trigger identity work but may not replace `artifacts_ledger` |
| `wake_artifact_apply_ledger` | `TARGET_DEPLOYMENT_LIFECYCLE` | target deployment history authority | authoritative write | `WakeTargetDeploymentWorker` only | sync worker, PH1.J, operators | artifact-global publication / revocation / replacement truth | append-only per-device history keyed by `(device_id, artifact_version, state, idempotency_key)` |
| `wake_artifact_apply_current` | `TARGET_DEPLOYMENT_LIFECYCLE` | target deployment current authority | authoritative write | `WakeTargetDeploymentWorker` only | sync worker, operators, runtime-use readers | artifact identity authority or runtime-use authority | `last_known_good_artifact_version` is the compensation anchor for rollback |
| `WakeRuntimeEventRecord` / `wake_runtime_events` | `RUNTIME_USE_LIFECYCLE` | runtime-use evidence authority | evidence only | existing PH1.W runtime event writer, not C2 workers | C2 workers read-only, PH1.J, PH1.GOV, PH1.LAW, operators | target deployment activation truth or artifact-global lifecycle truth | accepted/rejected wake events may inform visibility or later rollback requests, never reverse-authority mutation |
| `device_artifact_sync_*` queue surfaces | operational receipt/control support surface | queue-control authority only | receipt/projection/evidence only | `WakeSyncReceiptDeliveryWorker` only | operators, sync worker metrics, deployment worker read-only | artifact identity truth, target deployment truth, runtime-use truth | `Acked`, retry, replay-due, and dead-letter are queue state only |
| `audit.audit_events` | evidence overlay across planes | proof/audit authority only | evidence only | PH1.J append writer | operators, PH1.GOV, PH1.LAW, incident review | lifecycle state store | `STATE_TRANSITION` proves recorded evidence, not storage commit |
| `gov_decision_bundle` | governed decision input | decision-only authority | evidence/decision only | PH1.GOV | identity worker, promotion worker, PH1.LAW | committed lifecycle row | governs allow/block posture before protected commit |
| `RuntimeExecutionEnvelope.law_state` | final runtime law input | decision-only authority | evidence/decision only | PH1.LAW | identity worker, deployment worker, operators | committed lifecycle row | protected completion may depend on this posture without making it a lifecycle writer |
| `os_decision_bundle` | orchestration input | decision-only authority | evidence/decision only | PH1.OS | worker dispatch/orchestration only | artifact trust proof, lifecycle state, or storage current row | orchestration legality does not substitute for transition commit |

Reverse-Authority Refusal Rules
- Runtime-use evidence must never be read backward into target deployment truth or artifact identity truth.
- Rollout projections must never be read backward into artifact-global lifecycle truth.
- Queue receipts, sender ACKs, retry rows, dead-letter rows, PH1.J audit rows, and local cache observations must never be treated as lifecycle commits.
- Device-local success may prove local observation only. It may not rewrite cloud-authoritative artifact identity truth.
- On any cross-surface conflict, the worker must re-read the authoritative plane-owned row, preserve the lower-authority surface as evidence only, and refuse any reverse mutation.

Reverse-Authority Refusal Matrix
| source surface | tempting but forbidden inference | why forbidden | correct authoritative source | worker behavior on conflict |
| --- | --- | --- | --- | --- |
| `WakeRuntimeEventRecord` / `wake_runtime_events` | “runtime accepted this artifact, so target deployment must be active” | runtime-use is downstream consumption evidence and can exist only after or beside deployment truth; it is not the deployment writer | `wake_artifact_apply_ledger` and `wake_artifact_apply_current` | keep runtime evidence as visibility only; re-read deployment rows and refuse reverse mutation |
| `wake_promotion_current` | “rollout pointer says Active, so artifact-global identity is already ACTIVE” | rollout projection is bridge scope only; artifact identity still requires append-only artifact-global commit | `os_process.artifacts_ledger` | schedule or re-run identity append if missing; do not treat projection as final artifact-global truth |
| `wake_promotion_ledger` | “promotion history proves artifact-global replacement already happened” | promotion history records rollout events, not artifact-global lifecycle authority | `os_process.artifacts_ledger` | preserve ledger as rollout evidence only and refuse artifact-global inference without artifact append |
| `wake_artifact_apply_ledger` / `wake_artifact_apply_current` | “one device activated successfully, so artifact-global truth is ACTIVE for everyone” | target deployment is device-local or target-local only | `os_process.artifacts_ledger` | keep device-local truth local; if artifact identity conflicts, refuse upward rewrite and follow governed rollback/refusal path |
| `device_artifact_sync_*` queue rows | “ACKed sync means the lifecycle transition is complete” | queue state reflects send/control flow only | plane-owned lifecycle row for the affected subject | ACK, retry, or dead-letter must not mutate lifecycle truth; worker may only update queue state |
| `audit.audit_events` / PH1.J proof rows | “a `STATE_TRANSITION` audit row exists, so the lifecycle write must exist” | PH1.J is append-only evidence, not a parallel lifecycle state store | plane-owned authoritative row in `artifacts_ledger`, `wake_promotion_*`, or `wake_artifact_apply_*` | reconcile audit from authoritative state, never authoritative state from audit |
| local pull/apply observation in `device_artifact_sync` | “downloaded payload and local cache write prove artifact-global publication or activation” | local sync observations are assist signals subordinate to cloud authority | `os_process.artifacts_ledger` for identity; `wake_artifact_apply_*` for target deployment | preserve observation as operational evidence only; if cloud state disagrees, fail closed and do not uplift local truth |

K) WORKER TRIGGERS, SCHEDULING, AND OWNERSHIP
- `WakeArtifactIdentityTransitionWorker`
  - trigger:
  - a governed promotion projection reaches `Active`
  - an explicit rollback projection reaches `RolledBack`
  - scheduling:
  - event-driven from promotion commits
  - ownership:
  - one logical subject at a time per `(scope_type, scope_id, artifact_type, artifact_version)`
- `WakePromotionProjectionWorker`
  - trigger:
  - explicit rollout request
  - explicit blocked-version revalidation
  - explicit rollback request
  - scheduling:
  - event-driven control-plane worker; never a hidden background guesser
  - ownership:
  - one projection subject per `artifact_version`
- `WakeTargetDeploymentWorker`
  - trigger:
  - device pull/apply pass across known devices
  - pull response updates for `WakePack`
  - activation hook failure or post-stage failure
  - scheduling:
  - existing repo default worker pass already uses:
  - dequeue batch max `16`
  - worker lease `30_000 ms`
  - default sync retry-after `30_000 ms`
  - default max attempts `5`
  - ownership:
  - one deployment subject per `(device_id, artifact_version)`
- `WakeSyncReceiptDeliveryWorker`
  - trigger:
  - queued sync jobs
  - replay-due rows after retry window
  - sender ACK / retryable NACK / fatal NACK
  - scheduling:
  - lease-based queue worker under the same sync worker pass
  - ownership:
  - one queue subject per `sync_job_id`

Worker Trigger vs Backstop Sweeper Law
- Every C2 worker family must have one explicit primary trigger and one explicit backstop sweeper. The sweeper may recover missed work only from already-authoritative rows or already-existing queue subjects; it may not invent new lifecycle intent.
- Cross-node duplicate mutation is forbidden. For each worker family, one cloud lease-owner may act on one subject at a time. Device clients, runtime-use writers, and PH1.J / PH1.GOV / PH1.LAW surfaces are never lease owners for lifecycle mutation.
- Where current repo truth already provides a lease surface, C2 must use it directly. Where the current slice exposes only subject rows plus idempotent commit guards, the safest planning interpretation is one logical cloud owner per subject with replay-safe re-entry rather than parallel best-effort workers.
- If the primary trigger is missed, the backstop sweeper must recover from the same subject scope and must use the same idempotency family as the original worker path.

Worker Trigger / Sweeper / Lease Matrix
| worker family | primary trigger | backstop sweeper | ownership / lease scope | duplicate-start protection | missed-trigger recovery rule | forbidden execution context |
| --- | --- | --- | --- | --- | --- | --- |
| `WakeArtifactIdentityTransitionWorker` | promotion projection reaches a governed artifact-identity commit point such as rollout `Active` or `RolledBack` | bounded reconciliation scan over existing `wake_promotion_current` subjects whose rollout state implies artifact identity work but whose append-only artifact row is missing | one cloud owner per `(scope_type, scope_id, artifact_type, artifact_version)` subject; never device-local | `artifacts_ledger` unique scope/version key plus idempotency dedupe | re-read `wake_promotion_current` and `artifacts_ledger`; append only the missing artifact row with the same idempotency family | device client, runtime-use writer, PH1.J, PH1.GOV, PH1.LAW, PH1.OS as lifecycle writer |
| `WakePromotionProjectionWorker` | explicit rollout request, explicit blocked-version revalidation, or explicit rollback request | bounded reconciliation over existing `wake_promotion_current` / `wake_promotion_ledger` rows only; never invent a new rollout request | one cloud owner per `artifact_version` rollout subject | `(artifact_version, to_state, idempotency_key)` plus current-state validation | if a command trigger is missed, recover only when an existing projection subject already shows incomplete or lagging rollout state | device client, runtime-use writer, `WakeTargetDeploymentWorker`, PH1.J/GOV/LAW/OS as lifecycle writer |
| `WakeTargetDeploymentWorker` | scheduled `run_device_artifact_pull_apply_pass_internal` over known devices plus concrete pull response updates for `WakePack` | the same deterministic pull/apply pass rerun on restart or retry window, re-reading `wake_artifact_apply_current` and device inventory | one cloud owner per `(device_id, artifact_version)` deployment subject; device performs apply mechanics but is never lifecycle authority | per-state idempotency tuples in `wake_artifact_apply_idempotency_index` plus current-row prerequisite checks | rerun against the same device/version subject; stage/activate/rollback no-op or resume from the last committed row | artifact identity writer, promotion writer, runtime-use writer, PH1.J/GOV/LAW/OS as lifecycle writer |
| `WakeSyncReceiptDeliveryWorker` | `device_artifact_sync_dequeue_batch` selecting `Queued` or replay-due `InFlight` rows | `device_artifact_sync_replay_due_rows(now)` after lease expiry | queue-row lease on `sync_job_id` using `worker_id` and `lease_expires_at`; current repo truth already provides this boundary | dequeue marks `InFlight`, increments attempts, and later ACK/fail/dead-letter commits require matching `worker_id` | expired leased rows become replay-due and may be dequeued again by the next lawful worker | device client, identity/promotion/deployment workers as queue-state writers outside the leased pass |
- C2 does not authorize:
- autonomous background revocation workers
- autonomous expiry workers
- archive / purge / restore workers
- any worker that writes memory lifecycle state

L) IDEMPOTENCY, ORDERING, REPLAY, AND COLLISION LAW
- `artifacts_ledger`
  - append-only
  - duplicate idempotent write returns the original `artifact_id`
  - duplicate `(scope_type, scope_id, artifact_type, artifact_version)` without matching idempotency is a hard conflict
- `wake_promotion_*`
  - promotion replay is deduped by `(artifact_version, to_state, idempotency_key)`
  - blocked versions may not advance until explicit revalidation occurs
  - active promotion requires `active_gate_passed=true`
  - invalid state transitions fail closed
- `wake_artifact_apply_*`
  - per-state replay is deduped by `(device_id, artifact_version, state, idempotency_key)`
  - activate requires the artifact to be staged first
  - rollback requires a staged or active source row to exist
  - activating an already-active version is deterministic no-op replay, not a duplicate state mutation
- sync delivery queue
  - ACK does not become lifecycle truth
  - retryable send failures schedule replay with bounded retry-after
  - fatal failures or attempt exhaustion dead-letter the queue item without mutating lifecycle truth
- current repo transition law that C2 must keep explicit:
- `wake_promotion_transition_allowed` currently permits only:
- `None -> Candidate`
- `None -> Blocked`
- `Candidate -> Shadow`
- `Shadow -> Canary`
- `Canary -> Active`
- `Active -> RolledBack`
- `Blocked -> Candidate`
- `RolledBack -> Candidate`
- `Any -> Blocked`
- anything else is a fail-closed contract violation

Idempotency / Dedup Domain Matrix
| worker family | transition or operation | idempotency key tuple | ordering basis | dedup domain | retry-safe behavior | stale or duplicate behavior |
| --- | --- | --- | --- | --- | --- | --- |
| `WakeArtifactIdentityTransitionWorker` | artifact-global wake identity append (`ACTIVE`, `REPLACED`, `ROLLED_BACK` as applicable) | `(scope_type, scope_id, artifact_type=\"WakePack\", artifact_version, idempotency_key)` | append order by monotonic `artifact_id` after governed rollout outcome is fixed | `os_process.artifacts_ledger` unique scope/version plus idempotency unique key | replay returns the original append result and must not create a second artifact-global mutation | same scope/version without matching idempotency is a hard conflict and must be refused |
| `WakePromotionProjectionWorker` | rollout transition commit | `(artifact_version, to_state, idempotency_key)` | current `wake_promotion_current.state`, then monotonic `promotion_event_id` | `wake_promotion_idempotency_index` and `wake_promotion_transition_allowed` | replay returns the current projection row with no second transition append | invalid `from -> to` or blocked-without-revalidation stays fail-closed |
| `WakePromotionProjectionWorker` | blocked-version revalidation | `(artifact_version, idempotency_key)` with the same `decision_ref` | blocked-flag presence plus `decision_ref` equality | `wake_promotion_revalidation_idempotency_index` | replay with the same `decision_ref` is a no-op success | mismatched `decision_ref` or revalidation against a non-blocked version is refused |
| `WakeTargetDeploymentWorker` | stage commit | `(device_id, artifact_version, WakeArtifactApplyState::Staged, stable_sync_key(\"wake_stage\", device_id:artifact_version:package_hash:base_idem[:failure_family]))` | latest `apply_event_id` for the device/version plus current staged pointer | `wake_artifact_apply_idempotency_index` for `Staged` rows | replay returns the existing staged row and preserves the same staged pointer | same device/version with a different stage key is a different deterministic attempt family and must not silently overwrite active truth |
| `WakeTargetDeploymentWorker` | activate commit | `(device_id, artifact_version, WakeArtifactApplyState::Active, stable_sync_key(\"wake_activate\", device_id:artifact_version:payload_hash:base_idem))` | `wake_artifact_apply_current.staged_artifact_version` must match, then append order by `apply_event_id` | `wake_artifact_apply_idempotency_index` for `Active` rows | replay returns current deployment truth; if already active, it is deterministic no-op replay | missing or mismatched staged pointer is refused fail-closed |
| `WakeTargetDeploymentWorker` | rollback commit | `(device_id, artifact_version, WakeArtifactApplyState::RolledBack, stable_sync_key(\"wake_rollback\", device_id:artifact_version:package_hash:base_idem[:failure_family]))` | latest staged/active source row and `last_known_good_artifact_version` | `wake_artifact_apply_idempotency_index` for `RolledBack` rows | replay returns the same rollback result and must not create a second rollback mutation | if no staged/active source row exists, rollback is refused |
| `WakeSyncReceiptDeliveryWorker` | sync send / ACK / retry / dead-letter handling | `(sync_job_id, receipt_ref, envelope.idempotency_key)` with queue-state commits keyed by `sync_job_id` | dequeue lease ownership, `attempt_count`, and `lease_expires_at` | single `device_artifact_sync_queue` row per `sync_job_id` | replay after restart or lease expiry resumes the same queue row without rewriting lifecycle truth | mismatched `worker_id`, ACK after a foreign lease, or fail/dead-letter outside `InFlight` is refused |

Collision / Recovery Matrix
| collision or recovery case | winning canonical outcome | losing or rejected outcome | what remains authoritative | compensation or retry behavior | visibility expectation |
| --- | --- | --- | --- | --- | --- |
| promotion vs replacement | the newly governed `Active` wake version becomes canonical `ACTIVE`; prior active wake version becomes historical `REPLACED` at artifact identity plane | concurrent competing candidate that did not win the governed `Active` slot remains non-active projection only or becomes `Blocked` | append-only artifact identity rows plus `wake_promotion_active_artifact_version` | no silent overwrite; device rollout proceeds only from the winning active version; prior target deployments remain until replaced or rolled back explicitly | PH1.J audit required; PH1.GOV / PH1.LAW visible when enabled and safety-relevant |
| apply vs rollback | explicit rollback of the same device/version wins once rollback is committed; last-known-good is restored | in-flight or repeated activation of the losing device/version is rejected or becomes historical receipt-only evidence | `wake_artifact_apply_current` plus append-only apply ledger rows | rollback is the compensation path; replay with the same idempotency key returns the already-committed rollback result | PH1.J visibility when lifecycle-significant; governed/law visibility when the protected path requires it |
| revoke vs apply-in-flight | upstream `REVOKED` artifact identity posture or governed deny wins | in-flight device apply may not elevate the same subject into active use | artifact identity revocation posture plus target-local non-active or rollback posture | fail closed; if staging already occurred, rollback or refuse activation and keep the receipt historical only | PH1.J required when revocation is lifecycle-significant; PH1.GOV / PH1.LAW visible where admissibility changes |
| expiry vs activate | upstream `EXPIRED` posture wins for the same artifact subject | late activation of the expired same subject is rejected | artifact identity expiry posture plus target-local non-active or rollback posture | fail closed; if activation already staged, compensate with rollback or refusal using the same idempotency family | PH1.J required where safety/compliance relevant; PH1.GOV / PH1.LAW visible when active use is denied |
| retry after partial worker failure | previously committed state or deterministic no-op replay wins | duplicate second mutation is rejected | the last committed row in `artifacts_ledger`, `wake_promotion_*`, or `wake_artifact_apply_*` remains authoritative | retry with the same idempotency key must return the same result; sync send retries use queue retry/dead-letter rules only | PH1.J event duplication is forbidden; retry behavior must remain auditable and replayable |

M) FAILURE, COMPENSATION, AND SAFE REFUSAL MODEL
- C2 worker families must fail closed under the following conditions:
- missing prerequisite state
- missing required governance or law posture when the transition is protected
- invalid promotion state transition
- blocked promotion version without explicit revalidation
- duplicate artifact identity scope/version conflict without matching idempotency
- missing device row
- malformed payload ref, package hash, or idempotency key
- C2 compensation model is explicit:
- payload download failure
  - target deployment worker stages operational evidence only as allowed by current repo path
  - then rolls back with `WAKE_ARTIFACT_REASON_DOWNLOAD_FAILED`
- payload hash mismatch
  - stage evidence
  - rollback with `WAKE_ARTIFACT_REASON_HASH_MISMATCH`
- activation hook failure
  - rollback with `WAKE_ARTIFACT_REASON_ACTIVATION_FAILED`
- sender retryable failure
  - schedule retry through queue failure commit
- sender fatal failure or exhausted attempts
  - dead-letter the queue item
- Safe refusal law for C2:
- refuse before write when the transition is structurally invalid
- compensate after write only when the current repo path already committed an intermediate operational step and must recover safely
- never compensate by inventing a new lifecycle state family
- never convert a receipt, queue outcome, or runtime event into artifact identity truth

Worker Re-entry and Checkpoint Law
- Worker re-entry must always begin by re-reading the last authoritative checkpoint from the owning surface, not by trusting in-memory progress, prior local cache writes, or runtime-use evidence.
- C2 checkpoints are limited to plane-owned commits already present in repo truth:
- `artifacts_ledger` append rows for artifact identity
- `wake_promotion_ledger` plus `wake_promotion_current` for rollout projection
- `wake_artifact_apply_ledger` plus `wake_artifact_apply_current` for target deployment
- `device_artifact_sync_queue` row state, `attempt_count`, `worker_id`, and `lease_expires_at` for receipt delivery
- If the authoritative checkpoint exists and a later receipt, audit event, or projection update is missing, re-entry may emit the missing non-authoritative side effect idempotently but may not replay a second authoritative mutation.
- If only an intermediate checkpoint exists, re-entry may attempt only the next lawful transition from that checkpoint and must reuse the same subject scope and idempotency family.

Partial-Success / Re-entry Matrix
| case | authoritative state after failure | receipt / projection posture | worker re-entry rule | retry rule | compensation rule | proof / governance / law visibility expectation |
| --- | --- | --- | --- | --- | --- | --- |
| authoritative write succeeds, receipt emission fails | owning lifecycle row is already committed in `artifacts_ledger`, `wake_promotion_*`, or `wake_artifact_apply_*` | receipt or audit may be missing or lagging | re-read the authoritative row and emit only the missing receipt/audit side effect; never write the lifecycle row again | reuse the same idempotency family and correlation path | none unless a later authoritative conflict is detected | PH1.J may need idempotent backfill; PH1.GOV / PH1.LAW posture does not replay as a write |
| stage commit succeeds, activate commit fails | `wake_artifact_apply_ledger` contains `Staged`; `wake_artifact_apply_current.staged_artifact_version` remains set; active version is unchanged | local cache may exist; no active commit yet | re-enter at activation precondition check only; if staged pointer still matches, attempt activate, otherwise stop | retry the same activate key only after re-reading current staged subject | rollback if the failure family is download/hash/hook failure or if the staged subject is no longer lawful | PH1.J visible when lifecycle-significant; governance/law visibility only if protected activation is involved |
| apply commit succeeds, client acknowledgement is lost | `wake_artifact_apply_current.active_artifact_version` already points at the new version and `Active` append exists | client ACK or external sync receipt is missing | do not re-activate; emit missing receipt or queue work only | same activate key returns current state; queue replay uses the same `sync_job_id` family | none at C2 scope | PH1.J should reflect the committed apply once; governance/law visibility follows the committed activation path, not the missing ACK |
| projection update lags authoritative state | authoritative artifact or deployment row is already committed | `wake_promotion_current`, PH1.J, or operational projection is stale | sweeper reads the authoritative row and advances only the lagging non-authoritative surface | retry only the lagging projection/evidence update | none; reverse-authority compensation is forbidden | PH1.J backfill is allowed idempotently; governance/law consumers remain read-only |
| sync delivery retries after process restart | lifecycle authority is unchanged; queue row remains the control checkpoint | queue row is `InFlight` with expired lease or retry-due | dequeue only after lawful lease expiry and resume from the same `sync_job_id` | resend the same envelope identity with bounded attempts | none; delivery retry must not mutate lifecycle truth | PH1.J / GOV / LAW visibility remains tied to prior authoritative state, not the restarted send attempt |
| dead-letter record exists but authoritative state remains unchanged | lifecycle authority is unchanged; queue row is `DeadLetter` only | delivery evidence terminated locally; no new authoritative state | no automatic lifecycle mutation is permitted on re-entry | no automatic retry after dead-letter in C2 | none; manual requeue or broader escalation is outside the worker’s automatic path | PH1.J may record the delivery failure if lifecycle-significant; broader law/governance escalation remains deferred unless a protected path requires C4 treatment |

Dead-Letter, Quarantine, and Manual Review Boundaries
- C2 may define worker-local retry, dead-letter, and safe-refusal posture for queue/control failures and structurally invalid lifecycle attempts.
- C2 may not silently absorb full proof/governance/law redesign. When a failure requires cross-subsystem legal, proof, or governance reinterpretation, the worker must stop at a bounded refusal boundary and defer broader normalization to C4.
- Dead-letter is for worker-local control flow that exhausted safe retries or received fatal failure.
- Quarantine or manual review is required when the current slice exposes a protected-path inconsistency, conflicting authoritative inputs, corrupted queue control state, or repeated blocked subjects that cannot be resolved by replaying the same lawful idempotent operation.

Dead-Letter / Quarantine Boundary Matrix
| failure class | worker family | immediate action | authoritative state impact | retry allowed or not | dead-letter threshold / boundary | quarantine or manual review trigger | downstream defer-to-C4 note if applicable |
| --- | --- | --- | --- | --- | --- | --- | --- |
| retryable sender or transport failure below max attempts | `WakeSyncReceiptDeliveryWorker` | `device_artifact_sync_fail_commit` with bounded `retry_after_ms` | none; lifecycle rows stay unchanged | YES | dead-letter only when attempts hit the configured max or a later fatal error occurs | none by default | NO; this remains worker-local C2 control flow |
| fatal sender NACK or attempt exhaustion | `WakeSyncReceiptDeliveryWorker` | `device_artifact_sync_dead_letter_commit` | none; lifecycle rows stay unchanged | NO automatic retry | immediate on fatal NACK or at max-attempt boundary | manual review if delivery evidence matters operationally | NO, unless a protected downstream closure demands broader law/proof escalation outside C2 |
| blocked promotion without explicit revalidation | `WakePromotionProjectionWorker` | safe refusal; keep projection blocked | `wake_promotion_*` remains `Blocked` or unchanged | NO until explicit revalidation | no dead-letter; this is a structural refusal, not a send failure | manual review when repeated blocked subjects indicate unresolved rollout eligibility | NO; the bounded remedy is explicit revalidation inside current C2 scope |
| protected activation lacks required governance or law allow posture | `WakeArtifactIdentityTransitionWorker` or `WakeTargetDeploymentWorker` | refuse commit before write | authoritative lifecycle row remains unchanged | NO until a new lawful decision bundle exists | no worker-local dead-letter for the lifecycle subject itself | manual review when protected path inputs remain conflicting or incomplete | YES; broader proof/governance/law normalization remains a C4 concern |
| queue row corruption or worker-id mismatch on ACK/fail/dead-letter | `WakeSyncReceiptDeliveryWorker` | refuse the state change and preserve current row for inspection | none; lifecycle rows stay unchanged | NO until row integrity is corrected | no further automatic queue mutation from the conflicting worker context | quarantine/manual review because cross-worker ownership has become ambiguous | NO for lifecycle redesign; this is a bounded control-plane integrity problem |
| runtime-use evidence conflicts with current deployment or artifact identity posture | `WakeTargetDeploymentWorker` or `WakeArtifactIdentityTransitionWorker` | refuse reverse-authority mutation and preserve runtime evidence as evidence only | authoritative lifecycle row remains whichever higher-authority row already exists | NO automatic retry of the forbidden inference | no dead-letter for lifecycle state; if a queue job caused the conflict, dead-letter only the queue job | manual review when repeated evidence conflicts point to a broader protected-path inconsistency | YES when cross-subsystem law/proof interpretation is required beyond the bounded C2 refusal |

N) PROOF / GOVERNANCE / LAW VISIBILITY AT C2 SCOPE
- PH1.J
  - C2 lifecycle-significant state changes must be reason-coded and audit-visible
  - PH1.J remains append-only evidence only
  - C2 must not create a second proof system
  - per-artifact `proof_entry_ref` remains canonical when proof-required artifact authority is involved
- PH1.GOV
  - governed activation / deprecation / rollback posture must be consumed before protected wake artifact identity transitions are committed
  - PH1.GOV writes no lifecycle rows in C2
  - C2 must consume `gov_decision_bundle` and governance execution state, not bypass them
- PH1.LAW
  - protected completion remains unlawful without final PH1.LAW judgment where that path applies
  - PH1.LAW writes no lifecycle rows in C2
  - C2 may surface law-relevant lifecycle changes; C2 may not become a parallel law engine
- PH1.OS
  - remains orchestration-only
  - must not verify artifact trust, signatures, trust roots, lineage, or scope for wake artifacts
- C2 explicit boundary:
- C2 handles only the worker-scope visibility needed to keep wake artifact lifecycle safe and auditable
- C2 does not redesign end-to-end lifecycle proof/governance/law integration
- that broader normalization remains deferred to C4

O) OBSERVABILITY AND OPERATIONS
- Existing repo metrics and counters that C2 must preserve and use operationally:
- `DeviceArtifactSyncWorkerPassMetrics`
  - `dequeued_count`
  - `acked_count`
  - `retry_scheduled_count`
  - `dead_lettered_count`
  - `pulled_device_count`
  - `pulled_update_count`
  - `apply_activated_count`
  - `apply_rollback_count`
  - `apply_noop_count`
  - `pull_error_count`
  - `queue_after`
- existing queue visibility:
  - queued
  - in-flight
  - acked
  - dead-letter
  - replay-due
- C2 operational visibility must also make the following derivable from current authoritative rows:
- current wake rollout active artifact version
- blocked artifact versions awaiting revalidation
- per-device active artifact version
- per-device last-known-good version
- per-device rollback reason
- runtime-use evidence count by active wake artifact version
- C2 operations hard rule:
- operators may inspect queue, promotion, apply, and runtime-use evidence surfaces
- operators may not treat operational queue state or runtime events as artifact identity truth

P) EXPLICIT NON-GOALS / DEFERRED TO C3 OR C4
C2 → Downstream Boundary Matrix
| concern | handled in C2 | deferred to C3 | deferred to C4 | rationale |
| --- | --- | --- | --- | --- |
| wake worker ownership and authoritative row separation across artifact identity vs target deployment | YES | NO | NO | this is the core purpose of C2 |
| memory retention / archive / purge / delete / restore semantics | NO | YES | NO | memory lifecycle is a separate owned domain and belongs to C3 |
| artifact archive / purge / restore lifecycle storage normalization | NO | NO | YES | C2 must not widen into broader lifecycle storage redesign |
| full lifecycle proof / governance / law linkage normalization across wake rows | NO | NO | YES | C2 consumes Phase A, PH1.GOV, and PH1.LAW outputs but does not redesign them |
| runtime-use lifecycle normalization beyond current `WakeRuntimeEventRecord` evidence | NO | NO | YES | C2 names runtime-use and preserves the boundary, but does not redesign its storage model |
| wake-specific artifact authenticity verification expansion at ingest / pull / apply / activation | NO | NO | YES | C2 may only consume existing trust/governance/law posture and must not invent a new verification path |
| memory-lifecycle downstream impact handling for wake artifacts | NO | YES | NO | downstream retention/delete/archive semantics are a C3 concern |
- Additional explicit non-goals outside C2:
- no Android parity work
- no tests or verification closure work; that remains for C5
- no new trust path
- no new proof path
- no new governance path
- no new law path

Q) COMPLETION CRITERIA
- C2 is complete only when the implementation plan is explicit enough that later work cannot guess:
- which worker owns artifact identity writes
- which worker owns wake rollout projection writes
- which worker owns per-device deployment writes
- which surfaces are receipts only
- which surfaces are decision-only inputs
- C2 implementation closure must later prove:
- wake artifact identity truth lands in append-only `artifacts_ledger`
- wake rollout projection truth lands only in `wake_promotion_*`
- per-device deployment truth lands only in `wake_artifact_apply_*`
- runtime-use remains evidence-only at C2 scope
- invalid transitions fail closed
- replays return deterministic prior results
- rollback preserves `last_known_good`
- queue retries and dead-letter behavior do not mutate lifecycle truth
- PH1.J, PH1.GOV, PH1.LAW, and PH1.OS boundaries remain intact
- memory lifecycle remains untouched by C2 workers
- C2 is not complete if any of the following remain inferential:
- promotion projection vs artifact identity truth
- staged/apply/active/rollback ownership
- receipt vs authority separation
- runtime-use evidence vs activation truth
- C2 vs C3 vs C4 boundaries
