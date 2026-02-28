# Selene Engine Registry (Authoritative)

Purpose:
- Maintain the authoritative engine inventory for current Option B scope.
- Point each engine to its canonical DB wiring and ECM contracts.

Status source:
- Engine completion state is tracked in `docs/COVERAGE_MATRIX.md`.
- Design lock status is tracked in `docs/11_DESIGN_LOCK_SEQUENCE.md`.
- Phase C lock pass for `PH1.K/W/C/NLP/D/X` is recorded in `docs/COVERAGE_MATRIX.md` (Phase C Verification Pass).

## Rules

- Engines never call engines directly; Selene OS orchestrates.
- Any side effects are simulation-gated (`No Simulation -> No Execution`).
- Do not duplicate simulation lists here (use `docs/08_SIMULATION_CATALOG.md`).
- Do not duplicate blueprint records here (use `docs/09_BLUEPRINT_REGISTRY.md` and `docs/BLUEPRINTS/*.md`).

## OS Execution Law

### Execution Boundary Law — Probabilistic Reasoning, Deterministic Execution
Selene must follow this architectural boundary:

Probabilistic Layer (Allowed):
- Language generation
- Reasoning
- Summarization
- Research
- Data analysis
- Document/photo explanation
- Connector read-only queries
- Tone/personality shaping

These may be model-driven and non-deterministic.

Deterministic Boundary (Mandatory):
- Intent -> dispatch classification
- Access control decisions
- Simulation execution
- State mutation
- Ledger writes
- Artifact activation
- Provider promotion/demotion
- Onboarding progression
- Message sending
- Any irreversible action

All execution must:
- Pass Access checks
- Require ACTIVE simulation IDs (when applicable)
- Be idempotent
- Be replay-safe
- Be auditable
- Fail closed on any inconsistency

Language may be probabilistic.
Execution must never be probabilistic.

## Registry (Current Scope)

Direction lock:
- This registry is the frozen engine list for current MVP scope under Option B.
- Planned engines listed below are in-scope but intentionally not marked DONE until their 4-pack is authored.

| engine_id | phase | layer | authority | primary_role | db_wiring | ecm |
|---|---|---|---|---|---|---|
| PH1.F | A Foundations | Storage | Authoritative | Persistence schema/migrations/invariants | `docs/DB_WIRING/PH1_F.md` | `docs/ECM/PH1_F.md` |
| PH1.J | A Foundations | Storage | Authoritative | Audit event contract + append-only proof trail | `docs/DB_WIRING/PH1_J.md` | `docs/ECM/PH1_J.md` |
| SELENE_OS_CORE_TABLES | A Foundations | Governance/Storage | Authoritative | WorkOrder/session/core orchestration persistence | `docs/DB_WIRING/SELENE_OS_CORE_TABLES.md` | `docs/ECM/SELENE_OS_CORE_TABLES.md` |
| PBS_TABLES | A Foundations | Governance | Authoritative | Blueprint registry tables and mappings | `docs/DB_WIRING/PBS_TABLES.md` | `docs/ECM/PBS_TABLES.md` |
| SIMULATION_CATALOG_TABLES | A Foundations | Governance | Authoritative | Simulation catalog persistence | `docs/DB_WIRING/SIMULATION_CATALOG_TABLES.md` | `docs/ECM/SIMULATION_CATALOG_TABLES.md` |
| ENGINE_CAPABILITY_MAPS_TABLES | A Foundations | Governance | Authoritative | Capability map persistence | `docs/DB_WIRING/ENGINE_CAPABILITY_MAPS_TABLES.md` | `docs/ECM/ENGINE_CAPABILITY_MAPS_TABLES.md` |
| ARTIFACTS_LEDGER_TABLES | A Foundations | Storage/Learning | Authoritative | Artifacts and cache persistence | `docs/DB_WIRING/ARTIFACTS_LEDGER_TABLES.md` | `docs/ECM/ARTIFACTS_LEDGER_TABLES.md` |
| PH1.L | B Identity/Access | Control | Authoritative | Session lifecycle state/timers | `docs/DB_WIRING/PH1_L.md` | `docs/ECM/PH1_L.md` |
| PH1.VOICE.ID | B Identity/Access | Perception | Authoritative | Speaker identity assertion | `docs/DB_WIRING/PH1_VOICE_ID.md` | `docs/ECM/PH1_VOICE_ID.md` |
| PH1.ACCESS.001_PH2.ACCESS.002 | B Identity/Access | Governance | Authoritative | Access gate + per-user permission truth | `docs/DB_WIRING/PH1_ACCESS_001_PH2_ACCESS_002.md` | `docs/ECM/PH1_ACCESS_001_PH2_ACCESS_002.md` |
| PH1.POLICY | B Identity/Access | Control | Authoritative (policy decision only) | Global Rule Base + Snapshot (prompt discipline; placement: ALWAYS_ON) | `docs/DB_WIRING/PH1_POLICY.md` | `docs/ECM/PH1_POLICY.md` |
| PH1.K | C Perception/NLP/X | Perception | Authoritative (I/O) | Voice runtime audio substrate | `docs/DB_WIRING/PH1_K.md` | `docs/ECM/PH1_K.md` |
| PH1.W | C Perception/NLP/X | Perception | Authoritative (wake) | Wake detection and capture boundaries | `docs/DB_WIRING/PH1_W.md` | `docs/ECM/PH1_W.md` |
| PH1.C | C Perception/NLP/X | Perception | Authoritative (transcript gate) | STT routing + transcript quality gate | `docs/DB_WIRING/PH1_C.md` | `docs/ECM/PH1_C.md` |
| PH1.NLP | C Perception/NLP/X | Understanding | Non-Authoritative | Deterministic intent/field draft extraction | `docs/DB_WIRING/PH1_NLP.md` | `docs/ECM/PH1_NLP.md` |
| PH1.D | C Perception/NLP/X | Understanding | Non-Authoritative | LLM contract boundary + validation | `docs/DB_WIRING/PH1_D.md` | `docs/ECM/PH1_D.md` |
| PH1.X | C Perception/NLP/X | Control | Authoritative (conversation move) | One next conversational directive per turn | `docs/DB_WIRING/PH1_X.md` | `docs/ECM/PH1_X.md` |
| PH1.WRITE | D Output | Output | Non-Authoritative | Presentation-only formatting with critical-token/refusal-safe fallback | `docs/DB_WIRING/PH1_WRITE.md` | `docs/ECM/PH1_WRITE.md` |
| PH1.TTS | D Output | Output | Authoritative (playback) | Speech rendering with cancel safety | `docs/DB_WIRING/PH1_TTS.md` | `docs/ECM/PH1_TTS.md` |
| PH1.E | E Onboarding/Tools | Control | Authoritative (read-only tools) | Tool routing for read-only queries | `docs/DB_WIRING/PH1_E.md` | `docs/ECM/PH1_E.md` |
| PH1.BCAST | E Onboarding/Tools | Control | Authoritative (broadcast lifecycle only) | Broadcast lifecycle orchestrator (draft/deliver/ack/defer/retry/expire; active implementation ids: `PH1.BCAST.001`; includes canonical BCAST.MHP state machine for phone-first message handling; simulation-gated for external delivery; placement: TURN_OPTIONAL) | `docs/DB_WIRING/PH1_BCAST.md` | `docs/ECM/PH1_BCAST.md` |
| PH1.DELIVERY | E Onboarding/Tools | Control | Authoritative (delivery attempt truth only) | Provider gateway for SMS/Email/WhatsApp/WeChat delivery attempts (simulation-gated; placement: TURN_OPTIONAL) | `docs/DB_WIRING/PH1_DELIVERY.md` | `docs/ECM/PH1_DELIVERY.md` |
| PH1.LINK | E Onboarding/Tools | Control/Governance | Authoritative | Invite link lifecycle + selector-hint capture only (no schema ownership; simulation-gated) | `docs/DB_WIRING/PH1_LINK.md` | `docs/ECM/PH1_LINK.md` |
| PH1.ONB | E Onboarding/Tools | Governance/Control | Authoritative | Onboarding execution (deterministic one-question runner); executes pinned requirements schema only | `docs/DB_WIRING/PH1_ONB.md` | `docs/ECM/PH1_ONB.md` |
| PH1.REM | E Onboarding/Tools | Control | Authoritative (reminder timing state machine) | Deterministic reminder scheduling/delivery timing mechanics (includes BCAST.MHP follow-up timing handoff; simulation-gated) | `docs/DB_WIRING/PH1_REM.md` | `docs/ECM/PH1_REM.md` |
| PH1.POSITION | E Onboarding/Tools | Governance/Storage | Authoritative | Position lifecycle + position requirements schema ownership (versioned, auditable) | `docs/DB_WIRING/PH1_POSITION.md` | `docs/ECM/PH1_POSITION.md` |
| PH1.M | F Memory/Learning | Storage/Learning | Non-Authoritative | Memory Engine vNext (atoms + retrieval + emotional threads + device persistence via Engine B; canonical narrative: `docs/12_MEMORY_ARCHITECTURE.md`) | `docs/DB_WIRING/PH1_M.md` | `docs/ECM/PH1_M.md` |
| PH1.PERSONA | F Memory/Learning | Learning | Non-Authoritative | Identity-verified user style/tone/delivery profile hints | `docs/DB_WIRING/PH1_PERSONA.md` | `docs/ECM/PH1_PERSONA.md` |
| PH1.CAPREQ | G Capability Requests | Governance/Control | Authoritative (simulation-gated lifecycle) | Capability request lifecycle family state machine (active implementation ids: `PH1.CAPREQ.001`) | `docs/DB_WIRING/PH1_CAPREQ.md` | `docs/ECM/PH1_CAPREQ.md` |
| PH1.TENANT | H Enterprise Support | Governance/Control | Authoritative (tenant context decision) | Tenant/org context resolver for `tenant_id` + policy pointers (`TENANT_POLICY_EVALUATE -> TENANT_DECISION_COMPUTE`) | `docs/DB_WIRING/PH1_TENANT.md` | `docs/ECM/PH1_TENANT.md` |
| PH1.GOV | H Enterprise Support | Governance/Control | Authoritative (definition governance decision) | Governance decision engine for artifact activation/deprecation/rollback (`GOV_POLICY_EVALUATE -> GOV_DECISION_COMPUTE`) | `docs/DB_WIRING/PH1_GOV.md` | `docs/ECM/PH1_GOV.md` |
| PH1.QUOTA | H Enterprise Support | Governance/Control | Authoritative (quota lane decision) | Quota and budget lane gate for `ALLOW | WAIT | REFUSE` decisions (`QUOTA_POLICY_EVALUATE -> QUOTA_DECISION_COMPUTE`) | `docs/DB_WIRING/PH1_QUOTA.md` | `docs/ECM/PH1_QUOTA.md` |
| PH1.WORK | H Enterprise Support | Governance/Storage | Authoritative (work-order ledger decision) | Work-order append/no-op decision gate for append-only/idempotent replay-safe ledger writes (`WORK_POLICY_EVALUATE -> WORK_DECISION_COMPUTE`) | `docs/DB_WIRING/PH1_WORK.md` | `docs/ECM/PH1_WORK.md` |
| PH1.LEASE | H Enterprise Support | Governance/Control | Authoritative (lease ownership decision) | Work-order lease ownership decision gate for deterministic `ACQUIRE | RENEW | RELEASE` posture (`LEASE_POLICY_EVALUATE -> LEASE_DECISION_COMPUTE`) | `docs/DB_WIRING/PH1_LEASE.md` | `docs/ECM/PH1_LEASE.md` |
| PH1.OS | H Enterprise Support | Governance/Control | Authoritative (orchestration gate decision) | Selene OS orchestration gate for deterministic next-move + dispatch legality (`OS_POLICY_EVALUATE -> OS_DECISION_COMPUTE`) plus one canonical top-level turn slice (voice/text ALWAYS_ON sequence lock + TURN_OPTIONAL ordering from one control point + explicit per-turn optional budget contract enforcement + machine-only optional utility scoring gates `U4/U5` with deterministic `KEEP/DEGRADE/DISABLE_CANDIDATE` posture + fail-closed runtime-boundary guard for OFFLINE/control-plane engine ids) | `docs/DB_WIRING/PH1_OS.md` | `docs/ECM/PH1_OS.md` |
| PH1.HEALTH | H Enterprise Support | Observability/Control | Non-Authoritative (display-only) | Health dashboard/report projection for issue list, timeline, unresolved/escalated state visibility (no remediation execution in v1) | `docs/DB_WIRING/PH1_HEALTH.md` | `docs/ECM/PH1_HEALTH.md` |
| PH1.SCHED | H Enterprise Support | Governance/Control | Authoritative (deterministic scheduler decision) | Deterministic retry/wait/fail decision engine (`SCHED_POLICY_EVALUATE -> SCHED_DECISION_COMPUTE`) | `docs/DB_WIRING/PH1_SCHED.md` | `docs/ECM/PH1_SCHED.md` |
| PH1.EXPORT | H Enterprise Support | Governance/Storage | Authoritative (compliance proof export) | Compliance export proof generation (`EXPORT_ACCESS_EVALUATE -> EXPORT_ARTIFACT_BUILD`) with deterministic redaction + tamper-evident hash output | `docs/DB_WIRING/PH1_EXPORT.md` | `docs/ECM/PH1_EXPORT.md` |
| PH1.KMS | H Enterprise Support | Governance/Storage | Authoritative (secret material lifecycle) | Secret/key access evaluation + opaque handle issuance/rotation/revoke (`KMS_ACCESS_EVALUATE -> KMS_MATERIAL_ISSUE`) | `docs/DB_WIRING/PH1_KMS.md` | `docs/ECM/PH1_KMS.md` |

Storage grouping (non-runtime; not a callable engine row):
- `PH1.LEARN_FEEDBACK_KNOW` remains canonical as a persistence grouping contract only:
  - `docs/DB_WIRING/PH1_LEARN_FEEDBACK_KNOW.md`
  - `docs/ECM/PH1_LEARN_FEEDBACK_KNOW.md`
- Runtime owners are split and concrete:
  - `PH1.FEEDBACK` (feedback signal audit rows)
  - `PH1.LEARN` (adaptation artifact rows)
  - `PH1.KNOW` (tenant vocabulary/pronunciation artifact rows)

## Phase C Extension Engines (Wiring Web Added)

All rows below are non-executing assist engines. Actions remain controlled by Selene OS + Access + Simulation.

| engine_id | phase | layer | authority | primary_role | placement (wiring class) | db_wiring | ecm |
|---|---|---|---|---|---|---|---|
| PH1.ENDPOINT | C Perception/NLP/X | Perception Assist | Non-Authoritative | Streaming endpoint boundary assist (`PH1.K` VAD windows -> selected hint for `PH1.C`) | TURN_OPTIONAL (after PH1.K, before PH1.C finalization) | `docs/DB_WIRING/PH1_ENDPOINT.md` | `docs/ECM/PH1_ENDPOINT.md` |
| PH1.LANG | C Perception/NLP/X | Understanding Assist | Non-Authoritative | Multilingual detection, segmentation, and response-language mapping for C/SRL/NLP | TURN_OPTIONAL (pre-intent normalization + ambiguity trigger) | `docs/DB_WIRING/PH1_LANG.md` | `docs/ECM/PH1_LANG.md` |
| PH1.PRON | C Perception/NLP/X | Speech Assist | Non-Authoritative | Pronunciation enrollment + lexicon-pack hints for PH1.TTS and robustness hints for PH1.VOICE.ID/PH1.W | TURN_OPTIONAL (pronunciation assist enabled) | `docs/DB_WIRING/PH1_PRON.md` | `docs/ECM/PH1_PRON.md` |
| PH1.SRL | C Perception/NLP/X | Understanding | Non-Authoritative | Post-STT semantic repair layer (deterministic shorthand normalization + code-switch-preserving frame output; no intent drift) | ALWAYS_ON (after PH1.C transcript_ok) | `docs/DB_WIRING/PH1_SRL.md` | `docs/ECM/PH1_SRL.md` |
| PH1.DOC | C Perception/NLP/X | Evidence Analyzer | Non-Authoritative | Read-only document evidence extraction | TURN_OPTIONAL (only when document evidence provided) | `docs/DB_WIRING/PH1_DOC.md` | `docs/ECM/PH1_DOC.md` |
| PH1.SUMMARY | C Perception/NLP/X | Evidence Analyzer | Non-Authoritative | Evidence-backed summary synthesis with citation validation for downstream context/understanding | TURN_OPTIONAL (only when summary is enabled and evidence bundle exists) | `docs/DB_WIRING/PH1_SUMMARY.md` | `docs/ECM/PH1_SUMMARY.md` |
| PH1.VISION | C Perception/NLP/X | Evidence Analyzer | Non-Authoritative | Opt-in visual evidence extraction (image/screenshot/diagram), evidence-backed only | TURN_OPTIONAL (only when visual evidence provided and vision is enabled) | `docs/DB_WIRING/PH1_VISION.md` | `docs/ECM/PH1_VISION.md` |
| PH1.PRUNE | C Perception/NLP/X | Conversation Assist | Non-Authoritative | Missing-field pruning for one-question clarify (`PH1.NLP` required fields -> `PH1.X` clarify target) | TURN_OPTIONAL (multiple required fields missing) | `docs/DB_WIRING/PH1_PRUNE.md` | `docs/ECM/PH1_PRUNE.md` |
| PH1.DIAG | C Perception/NLP/X | Conversation Assist | Non-Authoritative | Pre-directive consistency verification (intent/fields/confirmation/privacy/memory safety) | TURN_OPTIONAL (before PH1.X finalize) | `docs/DB_WIRING/PH1_DIAG.md` | `docs/ECM/PH1_DIAG.md` |
| PH1.SEARCH | C Perception/NLP/X | Planning Assist | Non-Authoritative | Unified query + evidence assist for PH1.E (query rewrite + source-ranked evidence interpretation with no intent drift) | TURN_OPTIONAL (read-only lookup intents) | `docs/DB_WIRING/PH1_SEARCH.md` | `docs/ECM/PH1_SEARCH.md` |
| PH1.COST | C Perception/NLP/X | Planning Assist | Non-Authoritative | Unified turn-policy pacing + budget guardrails: urgency tagging + delivery preference hints + per-user/day STT/LLM/TTS/TOOL route guardrails (deterministic degrade hints only) | TURN_OPTIONAL (when turn-policy guardrails are enabled) | `docs/DB_WIRING/PH1_COST.md` | `docs/ECM/PH1_COST.md` |
| PH1.PREFETCH | C Perception/NLP/X | Planning Assist | Non-Authoritative | Read-only prefetch candidate builder/prioritizer with bounded TTL + deterministic idempotency dedupe keys | TURN_OPTIONAL (prefetch policy enabled) | `docs/DB_WIRING/PH1_PREFETCH.md` | `docs/ECM/PH1_PREFETCH.md` |
| PH1.EXPLAIN | C Perception/NLP/X | Conversation Assist | Non-Authoritative | Reason-coded explanation packet generation | TURN_OPTIONAL (explicit why/how request) | `docs/DB_WIRING/PH1_EXPLAIN.md` | `docs/ECM/PH1_EXPLAIN.md` |
| PH1.LISTEN | F Memory/Learning | Learning Assist | Non-Authoritative | Active listening environment classification + capture/delivery adaptation hints | TURN_OPTIONAL (post-turn/next-turn adaptation window) | `docs/DB_WIRING/PH1_LISTEN.md` | `docs/ECM/PH1_LISTEN.md` |
| PH1.EMO.GUIDE | F Memory/Learning | Learning Assist | Non-Authoritative (tone policy only) | Emotional guidance style profile classifier (`DOMINANT | GENTLE` + bounded modifiers) | TURN_OPTIONAL (pre-response tone-planning window) | `docs/DB_WIRING/PH1_EMO_GUIDE.md` | `docs/ECM/PH1_EMO_GUIDE.md` |
| PH1.EMO.CORE | F Memory/Learning | Learning Assist | Non-Authoritative (tone/snapshot profile core) | Emotional snapshot/profile core (`EMO_SIM_001..006`: classify/reevaluate/privacy/tone/audit) | TURN_OPTIONAL (adaptation/onboarding tone-continuity window) | `docs/DB_WIRING/PH1_EMO_CORE.md` | `docs/ECM/PH1_EMO_CORE.md` |
| PH1.FEEDBACK | F Memory/Learning | Learning Assist | Non-Authoritative | Structured correction/confidence feedback capture and signal emission for LEARN/PAE | TURN_OPTIONAL (post-turn feedback window) | `docs/DB_WIRING/PH1_FEEDBACK.md` | `docs/ECM/PH1_FEEDBACK.md` |
| PH1.LEARN | F Memory/Learning | Learning Assist | Non-Authoritative | Learning signal aggregation + adaptation artifact package builder (`LEARN_SIGNAL_AGGREGATE -> LEARN_ARTIFACT_PACKAGE_BUILD`) | TURN_OPTIONAL (post-turn learning/adaptation window) | `docs/DB_WIRING/PH1_LEARN.md` | `docs/ECM/PH1_LEARN.md` |
| PH1.PAE | F Memory/Learning | Learning Assist | Non-Authoritative | Provider arbitration score build + adaptation hint emission (`PAE_POLICY_SCORE_BUILD -> PAE_ADAPTATION_HINT_EMIT`) | TURN_OPTIONAL (async adaptation window) | `docs/DB_WIRING/PH1_PAE.md` | `docs/ECM/PH1_PAE.md` |
| PH1.CACHE | F Memory/Learning | Learning Assist | Non-Authoritative | Cached decision-path skeleton build + refresh validation (`CACHE_HINT_SNAPSHOT_READ -> CACHE_HINT_SNAPSHOT_REFRESH`) | TURN_OPTIONAL (cache policy enabled) | `docs/DB_WIRING/PH1_CACHE.md` | `docs/ECM/PH1_CACHE.md` |
| PH1.KNOW | F Memory/Learning | Knowledge Assist | Non-Authoritative | Tenant dictionary and pronunciation-hint pack composition (`KNOW_DICTIONARY_PACK_BUILD -> KNOW_HINT_BUNDLE_SELECT`) | TURN_OPTIONAL (knowledge assist enabled) | `docs/DB_WIRING/PH1_KNOW.md` | `docs/ECM/PH1_KNOW.md` |
| PH1.MULTI | F Memory/Learning | Learning Assist | Non-Authoritative | Multimodal context fusion (voice/text + optional vision/doc evidence), privacy-scoped advisory output only | TURN_OPTIONAL (multi-hint fusion enabled) | `docs/DB_WIRING/PH1_MULTI.md` | `docs/ECM/PH1_MULTI.md` |
| PH1.CONTEXT | F Memory/Learning | Context Composition | Non-Authoritative | Bounded context bundle assembly + trim validation (`CONTEXT_BUNDLE_BUILD -> CONTEXT_BUNDLE_TRIM`) | ALWAYS_ON (before PH1.NLP/PH1.X) | `docs/DB_WIRING/PH1_CONTEXT.md` | `docs/ECM/PH1_CONTEXT.md` |
| PH1.KG | F Memory/Learning | Knowledge Assist | Non-Authoritative | Tenant-scoped relationship grounding hints (evidence-backed, no-guessing) | TURN_OPTIONAL (grounding requested) | `docs/DB_WIRING/PH1_KG.md` | `docs/ECM/PH1_KG.md` |
| PH1.PATTERN | F Memory/Learning | Offline Learning | Non-Authoritative | Offline pattern mining + proposal emission feeding PH1.RLL ranking | OFFLINE_ONLY (artifact proposal pipeline only) | `docs/DB_WIRING/PH1_PATTERN.md` | `docs/ECM/PH1_PATTERN.md` |
| PH1.RLL | F Memory/Learning | Offline Learning | Non-Authoritative | Offline RL ladder ranking for governed artifact recommendations (Tier-3 approval required) | OFFLINE_ONLY (artifact proposal pipeline only) | `docs/DB_WIRING/PH1_RLL.md` | `docs/ECM/PH1_RLL.md` |

## Maintenance
When an engine is added, removed, split, or renamed:
1. Update this registry.
2. Update `docs/COVERAGE_MATRIX.md`.
3. Add/update matching `docs/DB_WIRING/*.md` and `docs/ECM/*.md`.
4. Update simulation/blueprint canon if side effects or workflows changed.

PH1.M narrative contract:
- `docs/12_MEMORY_ARCHITECTURE.md`
