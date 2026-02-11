# Selene Engine Registry (Authoritative)

Purpose:
- Maintain the authoritative engine inventory for current Option B scope.
- Point each engine to its canonical DB wiring and ECM contracts.

Status source:
- Engine completion state is tracked in `docs/COVERAGE_MATRIX.md`.
- Design lock status is tracked in `docs/11_DESIGN_LOCK_SEQUENCE.md`.

## Rules

- Engines never call engines directly; Selene OS orchestrates.
- Any side effects are simulation-gated (`No Simulation -> No Execution`).
- Do not duplicate simulation lists here (use `docs/08_SIMULATION_CATALOG.md`).
- Do not duplicate blueprint records here (use `docs/09_BLUEPRINT_REGISTRY.md` and `docs/BLUEPRINTS/*.md`).

## Registry (Current Scope)

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
| PH1.K | C Perception/NLP/X | Perception | Authoritative (I/O) | Voice runtime audio substrate | `docs/DB_WIRING/PH1_K.md` | `docs/ECM/PH1_K.md` |
| PH1.W | C Perception/NLP/X | Perception | Authoritative (wake) | Wake detection and capture boundaries | `docs/DB_WIRING/PH1_W.md` | `docs/ECM/PH1_W.md` |
| PH1.C | C Perception/NLP/X | Perception | Authoritative (transcript gate) | STT routing + transcript quality gate | `docs/DB_WIRING/PH1_C.md` | `docs/ECM/PH1_C.md` |
| PH1.NLP | C Perception/NLP/X | Understanding | Non-Authoritative | Deterministic intent/field draft extraction | `docs/DB_WIRING/PH1_NLP.md` | `docs/ECM/PH1_NLP.md` |
| PH1.D | C Perception/NLP/X | Understanding | Non-Authoritative | LLM contract boundary + validation | `docs/DB_WIRING/PH1_D.md` | `docs/ECM/PH1_D.md` |
| PH1.X | C Perception/NLP/X | Control | Authoritative (conversation move) | One next conversational directive per turn | `docs/DB_WIRING/PH1_X.md` | `docs/ECM/PH1_X.md` |
| PH1.WRITE | D Output | Output | Non-Authoritative | Presentation-only formatting | `docs/DB_WIRING/PH1_WRITE.md` | `docs/ECM/PH1_WRITE.md` |
| PH1.TTS | D Output | Output | Authoritative (playback) | Speech rendering with cancel safety | `docs/DB_WIRING/PH1_TTS.md` | `docs/ECM/PH1_TTS.md` |
| PH1.E | E Onboarding/Tools | Control | Authoritative (read-only tools) | Tool routing for read-only queries | `docs/DB_WIRING/PH1_E.md` | `docs/ECM/PH1_E.md` |
| PH1.LINK | E Onboarding/Tools | Control/Governance | Authoritative | Invite link lifecycle (simulation-gated) | `docs/DB_WIRING/PH1_LINK.md` | `docs/ECM/PH1_LINK.md` |
| PH1.ONB | E Onboarding/Tools | Governance/Control | Authoritative | Onboarding orchestration | `docs/DB_WIRING/PH1_ONB.md` | `docs/ECM/PH1_ONB.md` |
| PH1.POSITION | E Onboarding/Tools | Governance/Storage | Authoritative | Position truth lifecycle | `docs/DB_WIRING/PH1_POSITION.md` | `docs/ECM/PH1_POSITION.md` |
| PH1.M | F Memory/Learning | Storage/Learning | Non-Authoritative | Memory retrieval/storage with evidence policy | `docs/DB_WIRING/PH1_M.md` | `docs/ECM/PH1_M.md` |
| PH1.PERSONA | F Memory/Learning | Learning | Non-Authoritative | User style/tone profile hints | `docs/DB_WIRING/PH1_PERSONA.md` | `docs/ECM/PH1_PERSONA.md` |
| PH1.LEARN_FEEDBACK_KNOW | F Memory/Learning | Learning | Non-Authoritative | Feedback + learning artifacts + knowledge packs | `docs/DB_WIRING/PH1_LEARN_FEEDBACK_KNOW.md` | `docs/ECM/PH1_LEARN_FEEDBACK_KNOW.md` |
| PH1.CAPREQ | G Capability Requests | Governance/Control | Authoritative (simulation-gated lifecycle) | Capability request lifecycle state machine | `docs/DB_WIRING/PH1_CAPREQ.md` | `docs/ECM/PH1_CAPREQ.md` |

## Maintenance

When an engine is added, removed, split, or renamed:
1. Update this registry.
2. Update `docs/COVERAGE_MATRIX.md`.
3. Add/update matching `docs/DB_WIRING/*.md` and `docs/ECM/*.md`.
4. Update simulation/blueprint canon if side effects or workflows changed.
