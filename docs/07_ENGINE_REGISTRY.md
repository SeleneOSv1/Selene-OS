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
| PH1.WRITE | D Output | Output | Non-Authoritative | Presentation-only formatting | `docs/DB_WIRING/PH1_WRITE.md` | `docs/ECM/PH1_WRITE.md` |
| PH1.TTS | D Output | Output | Authoritative (playback) | Speech rendering with cancel safety | `docs/DB_WIRING/PH1_TTS.md` | `docs/ECM/PH1_TTS.md` |
| PH1.E | E Onboarding/Tools | Control | Authoritative (read-only tools) | Tool routing for read-only queries | `docs/DB_WIRING/PH1_E.md` | `docs/ECM/PH1_E.md` |
| PH1.BCAST | E Onboarding/Tools | Control | Authoritative (broadcast lifecycle only) | Broadcast lifecycle orchestrator (draft/deliver/ack/defer/retry/expire; includes canonical BCAST.MHP state machine for phone-first message handling; simulation-gated for external delivery; placement: TURN_OPTIONAL) | `docs/DB_WIRING/PH1_BCAST.md` | `docs/ECM/PH1_BCAST.md` |
| PH1.DELIVERY | E Onboarding/Tools | Control | Authoritative (delivery attempt truth only) | Provider gateway for SMS/Email/WhatsApp/WeChat delivery attempts (simulation-gated; placement: TURN_OPTIONAL) | `docs/DB_WIRING/PH1_DELIVERY.md` | `docs/ECM/PH1_DELIVERY.md` |
| PH1.ONBOARDING_SMS | E Onboarding/Tools | Control | Authoritative (setup lifecycle only) | SMS app setup verification/confirmation before any SMS send path (placement: TURN_OPTIONAL) | `docs/DB_WIRING/PH1_ONBOARDING_SMS.md` | `docs/ECM/PH1_ONBOARDING_SMS.md` |
| PH1.LINK | E Onboarding/Tools | Control/Governance | Authoritative | Invite link lifecycle (simulation-gated) | `docs/DB_WIRING/PH1_LINK.md` | `docs/ECM/PH1_LINK.md` |
| PH1.ONB | E Onboarding/Tools | Governance/Control | Authoritative | Onboarding execution (deterministic one-question runner); executes pinned requirements schema only | `docs/DB_WIRING/PH1_ONB.md` | `docs/ECM/PH1_ONB.md` |
| PH1.REM | E Onboarding/Tools | Control | Authoritative (reminder timing state machine) | Deterministic reminder scheduling/delivery timing mechanics (includes BCAST.MHP follow-up timing handoff; simulation-gated) | `docs/DB_WIRING/PH1_REM.md` | `docs/ECM/PH1_REM.md` |
| PH1.POSITION | E Onboarding/Tools | Governance/Storage | Authoritative | Position lifecycle + position requirements schema ownership (versioned, auditable) | `docs/DB_WIRING/PH1_POSITION.md` | `docs/ECM/PH1_POSITION.md` |
| PH1.M | F Memory/Learning | Storage/Learning | Non-Authoritative | Memory Engine vNext (atoms + retrieval + emotional threads + device persistence via Engine B; canonical narrative: `docs/12_MEMORY_ARCHITECTURE.md`) | `docs/DB_WIRING/PH1_M.md` | `docs/ECM/PH1_M.md` |
| PH1.PERSONA | F Memory/Learning | Learning | Non-Authoritative | User style/tone profile hints | `docs/DB_WIRING/PH1_PERSONA.md` | `docs/ECM/PH1_PERSONA.md` |
| PH1.LEARN_FEEDBACK_KNOW | F Memory/Learning | Learning | Non-Authoritative | Feedback + learning artifacts + knowledge packs | `docs/DB_WIRING/PH1_LEARN_FEEDBACK_KNOW.md` | `docs/ECM/PH1_LEARN_FEEDBACK_KNOW.md` |
| PH1.LEARNING_ADAPTIVE | F Memory/Learning | Learning | Non-Authoritative | Adaptive learning from draft/language feedback to improve future phrasing hints (placement: TURN_OPTIONAL async) | `docs/DB_WIRING/PH1_LEARNING_ADAPTIVE.md` | `docs/ECM/PH1_LEARNING_ADAPTIVE.md` |
| PH1.CAPREQ | G Capability Requests | Governance/Control | Authoritative (simulation-gated lifecycle) | Capability request lifecycle state machine | `docs/DB_WIRING/PH1_CAPREQ.md` | `docs/ECM/PH1_CAPREQ.md` |

## Phase C Extension Engines (Wiring Web Added)

All rows below are non-executing assist engines. Actions remain controlled by Selene OS + Access + Simulation.

| engine_id | phase | layer | authority | primary_role | placement (wiring class) | db_wiring | ecm |
|---|---|---|---|---|---|---|---|
| PH1.ENDPOINT | C Perception/NLP/X | Perception Assist | Non-Authoritative | Capture endpoint boundary assist | TURN_OPTIONAL (after PH1.K, before PH1.C finalization) | `docs/DB_WIRING/PH1_ENDPOINT.md` | `docs/ECM/PH1_ENDPOINT.md` |
| PH1.LANG | C Perception/NLP/X | Understanding Assist | Non-Authoritative | Multilingual detection, segmentation, and response-language mapping for C/SRL/NLP | TURN_OPTIONAL (pre-intent normalization + ambiguity trigger) | `docs/DB_WIRING/PH1_LANG.md` | `docs/ECM/PH1_LANG.md` |
| PH1.SRL | C Perception/NLP/X | Understanding | Non-Authoritative | Semantic role labeling scaffold | ALWAYS_ON (after PH1.C transcript_ok) | `docs/DB_WIRING/PH1_SRL.md` | `docs/ECM/PH1_SRL.md` |
| PH1.PUZZLE | C Perception/NLP/X | Understanding Assist | Non-Authoritative | Ambiguity candidate generation | TURN_OPTIONAL (after PH1.SRL when ambiguous) | `docs/DB_WIRING/PH1_PUZZLE.md` | `docs/ECM/PH1_PUZZLE.md` |
| PH1.ATTN | C Perception/NLP/X | Understanding Assist | Non-Authoritative | Attention weighting hints | TURN_OPTIONAL (NLP/context ranking assist) | `docs/DB_WIRING/PH1_ATTN.md` | `docs/ECM/PH1_ATTN.md` |
| PH1.DOC | C Perception/NLP/X | Evidence Analyzer | Non-Authoritative | Read-only document evidence extraction | TURN_OPTIONAL (only when document evidence provided) | `docs/DB_WIRING/PH1_DOC.md` | `docs/ECM/PH1_DOC.md` |
| PH1.VISION | C Perception/NLP/X | Evidence Analyzer | Non-Authoritative | Read-only image evidence extraction | TURN_OPTIONAL (only when image evidence provided) | `docs/DB_WIRING/PH1_VISION.md` | `docs/ECM/PH1_VISION.md` |
| PH1.PRUNE | C Perception/NLP/X | Conversation Assist | Non-Authoritative | Missing-field pruning for one-question clarify | TURN_OPTIONAL (multiple required fields missing) | `docs/DB_WIRING/PH1_PRUNE.md` | `docs/ECM/PH1_PRUNE.md` |
| PH1.DIAG | C Perception/NLP/X | Conversation Assist | Non-Authoritative | Pre-directive diagnostics | TURN_OPTIONAL (before PH1.X finalize) | `docs/DB_WIRING/PH1_DIAG.md` | `docs/ECM/PH1_DIAG.md` |
| PH1.SEARCH | C Perception/NLP/X | Planning Assist | Non-Authoritative | Search plan assist for PH1.E | TURN_OPTIONAL (read-only lookup intents) | `docs/DB_WIRING/PH1_SEARCH.md` | `docs/ECM/PH1_SEARCH.md` |
| PH1.WEBINT | C Perception/NLP/X | Evidence Analyzer | Non-Authoritative | Read-only web evidence interpretation | TURN_OPTIONAL (after PH1.E evidence returns) | `docs/DB_WIRING/PH1_WEBINT.md` | `docs/ECM/PH1_WEBINT.md` |
| PH1.PREFETCH | C Perception/NLP/X | Planning Assist | Non-Authoritative | Prefetch candidate suggestion | TURN_OPTIONAL (prefetch policy enabled) | `docs/DB_WIRING/PH1_PREFETCH.md` | `docs/ECM/PH1_PREFETCH.md` |
| PH1.EXPLAIN | C Perception/NLP/X | Conversation Assist | Non-Authoritative | Reason-coded explanation packet generation | TURN_OPTIONAL (explicit why/how request) | `docs/DB_WIRING/PH1_EXPLAIN.md` | `docs/ECM/PH1_EXPLAIN.md` |
| PH1.LISTEN | F Memory/Learning | Learning Assist | Non-Authoritative | Post-turn signal aggregation | TURN_OPTIONAL (post-turn telemetry window) | `docs/DB_WIRING/PH1_LISTEN.md` | `docs/ECM/PH1_LISTEN.md` |
| PH1.PAE | F Memory/Learning | Learning Assist | Non-Authoritative | Policy adaptation evaluation hints | TURN_OPTIONAL (async adaptation window) | `docs/DB_WIRING/PH1_PAE.md` | `docs/ECM/PH1_PAE.md` |
| PH1.CACHE | F Memory/Learning | Learning Assist | Non-Authoritative | Hint cache snapshot management | TURN_OPTIONAL (cache policy enabled) | `docs/DB_WIRING/PH1_CACHE.md` | `docs/ECM/PH1_CACHE.md` |
| PH1.MULTI | F Memory/Learning | Learning Assist | Non-Authoritative | Multi-source hint fusion | TURN_OPTIONAL (multi-hint fusion enabled) | `docs/DB_WIRING/PH1_MULTI.md` | `docs/ECM/PH1_MULTI.md` |
| PH1.CONTEXT | F Memory/Learning | Context Composition | Non-Authoritative | Bounded context bundle assembly | ALWAYS_ON (before PH1.NLP/PH1.X) | `docs/DB_WIRING/PH1_CONTEXT.md` | `docs/ECM/PH1_CONTEXT.md` |
| PH1.KG | F Memory/Learning | Knowledge Assist | Non-Authoritative | Entity/knowledge grounding hints | TURN_OPTIONAL (grounding requested) | `docs/DB_WIRING/PH1_KG.md` | `docs/ECM/PH1_KG.md` |
| PH1.PATTERN | F Memory/Learning | Offline Learning | Non-Authoritative | Offline pattern mining proposals | OFFLINE_ONLY (artifact proposal pipeline only) | `docs/DB_WIRING/PH1_PATTERN.md` | `docs/ECM/PH1_PATTERN.md` |
| PH1.RLL | F Memory/Learning | Offline Learning | Non-Authoritative | Offline ranking/learning loop proposals | OFFLINE_ONLY (artifact proposal pipeline only) | `docs/DB_WIRING/PH1_RLL.md` | `docs/ECM/PH1_RLL.md` |

## Planned In MVP Scope (Not Yet 4-Pack Finalized)

These engines are planned in current MVP design scope but do not yet have finalized DB_WIRING + ECM contracts.

| engine_id | phase | layer | authority | primary_role | db_wiring | ecm |
|---|---|---|---|---|---|---|
| PH1.EMO | F Memory/Learning | Learning | Non-Authoritative (tone-only) | Emotional continuity and tone guidance profile lifecycle | `docs/DB_WIRING/PH1_EMO.md` | `docs/ECM/PH1_EMO.md` |

## Maintenance
When an engine is added, removed, split, or renamed:
1. Update this registry.
2. Update `docs/COVERAGE_MATRIX.md`.
3. Add/update matching `docs/DB_WIRING/*.md` and `docs/ECM/*.md`.
4. Update simulation/blueprint canon if side effects or workflows changed.

PH1.M narrative contract:
- `docs/12_MEMORY_ARCHITECTURE.md`
