# Selene DB Ownership Matrix (Authoritative Summary)

Purpose:
- define which engines own writes for PH1.F tables
- prevent hidden cross-engine DB writes
- centralize ownership at summary level without duplicating DB wiring specs

Hard rules:
- All side-effect writes are simulation-gated (`DRAFT`/`COMMIT`/`REVOKE`).
- No engine may write outside declared ownership scope.
- `PH1.F` owns schema/migrations/invariants; domain engines own business writes only.
- Retries must be idempotent.
- Tenant isolation is mandatory.

Lock status source:
- Design-lock status is canonical in `docs/11_DESIGN_LOCK_SEQUENCE.md`.

Detailed contracts source:
- Table schemas/keys/constraints: `docs/04_KERNEL_CONTRACTS.md`
- Per-engine DB wiring contracts: `docs/DB_WIRING/*.md`

## Ownership Matrix (Summary)

| engine_id | write_scope_summary | db_wiring_doc |
|---|---|---|
| PH1.F | Schema/migration/invariant management only | `docs/DB_WIRING/PH1_F.md` |
| PH1.J | `audit_events` (append-only) | `docs/DB_WIRING/PH1_J.md` |
| SELENE_OS_CORE_TABLES | WorkOrder/session/core orchestration tables | `docs/DB_WIRING/SELENE_OS_CORE_TABLES.md` |
| PBS_TABLES | Blueprint registry tables | `docs/DB_WIRING/PBS_TABLES.md` |
| SIMULATION_CATALOG_TABLES | Simulation catalog tables | `docs/DB_WIRING/SIMULATION_CATALOG_TABLES.md` |
| ENGINE_CAPABILITY_MAPS_TABLES | ECM catalog tables | `docs/DB_WIRING/ENGINE_CAPABILITY_MAPS_TABLES.md` |
| ARTIFACTS_LEDGER_TABLES | Artifacts/tool-cache tables | `docs/DB_WIRING/ARTIFACTS_LEDGER_TABLES.md` |
| PH1.L | Session lifecycle state rows | `docs/DB_WIRING/PH1_L.md` |
| PH1.VOICE.ID | Voice enrollment/profile bindings (where applicable) | `docs/DB_WIRING/PH1_VOICE_ID.md` |
| PH1.ACCESS.001_PH2.ACCESS.002 | Access gate + schema lifecycle storage truth: access instances/overrides, AP schema ledger/current, AP authoring review ledger/current, AP rule-review action ledger, overlay/board policy ledgers/current, board-vote ledger, compile-lineage refs | `docs/DB_WIRING/PH1_ACCESS_001_PH2_ACCESS_002.md` |
| PH1.K | Audio runtime event/current projection tables + bounded VAD marker rows in conversation ledger | `docs/DB_WIRING/PH1_K.md` |
| PH1.W | Wake enrollment/runtime/profile tables + tuning snapshot references in audit payloads | `docs/DB_WIRING/PH1_W.md` |
| PH1.C | STT transcript ledger rows + provider-arbitration/evidence-span audit rows | `docs/DB_WIRING/PH1_C.md` |
| PH1.NLP | NLP intent/clarify/chat audit rows with required_fields/ambiguity metadata | `docs/DB_WIRING/PH1_NLP.md` |
| PH1.D | LLM-router audit rows with request envelope + model assignment snapshots | `docs/DB_WIRING/PH1_D.md` |
| PH1.X | Conversation-control audit rows keyed to WorkOrder/lease gating references | `docs/DB_WIRING/PH1_X.md` |
| PH1.WRITE | Presentation audit writes only | `docs/DB_WIRING/PH1_WRITE.md` |
| PH1.TTS | TTS audit/runtime rows as scoped | `docs/DB_WIRING/PH1_TTS.md` |
| PH1.E | Tool router audit/cache rows as scoped | `docs/DB_WIRING/PH1_E.md` |
| PH1.ONBOARDING_SMS | `comms.sms_app_setup_ledger/current` (SMS app setup lifecycle truth before SMS send) | `docs/DB_WIRING/PH1_ONBOARDING_SMS.md` |
| PH1.LINK | Onboarding draft/token lifecycle tables + selector-hint capture only (no schema definition ownership) | `docs/DB_WIRING/PH1_LINK.md` |
| PH1.ONB | Onboarding session execution tables only (ask/commit/complete); reads pinned requirements schema, does not own schema definitions | `docs/DB_WIRING/PH1_ONB.md` |
| PH1.POSITION | Position lifecycle + position requirements schema (versioned current + ledger ownership) | `docs/DB_WIRING/PH1_POSITION.md` |
| PH1.M | `memory_atoms_ledger` + `memory_atoms_current` (memory fact persistence and rebuildable current projection; see `docs/12_MEMORY_ARCHITECTURE.md`) | `docs/DB_WIRING/PH1_M.md` |
| PH1.M | `memory_suppression_rules` (`DO_NOT_MENTION | DO_NOT_REPEAT | DO_NOT_STORE`, targetable by `thread_id`/`work_order_id`/`topic_key`) | `docs/DB_WIRING/PH1_M.md` |
| PH1.M | `emotional_threads_ledger` + `emotional_threads_current` (tone-only continuity, non-authoritative) | `docs/DB_WIRING/PH1_M.md` |
| PH1.M | `memory_threads_ledger` + `memory_threads_current` + `memory_thread_refs` (thread digest continuity + bounded conversation pointers + tier policy fields `pinned/unresolved`) | `docs/DB_WIRING/PH1_M.md` |
| PH1.M | `memory_graph_nodes` + `memory_graph_edges` (non-authoritative graph retrieval index) | `docs/DB_WIRING/PH1_M.md` |
| PH1.M | `memory_archive_index` (optional pointer index for deterministic paging/page-in) | `docs/DB_WIRING/PH1_M.md` |
| PH1.M | `memory_metrics_ledger` (Q12-MEM..Q18-MEM quality telemetry; non-authoritative) | `docs/DB_WIRING/PH1_M.md` |
| PH1.PERSONA | Persona-related bounded writes (if present) | `docs/DB_WIRING/PH1_PERSONA.md` |
| PH1.LEARN_FEEDBACK_KNOW | Learning feedback/artifact tables as scoped | `docs/DB_WIRING/PH1_LEARN_FEEDBACK_KNOW.md` |
| PH1.LEARNING_ADAPTIVE | `learning.adaptive_feedback_ledger/current`, `learning.adaptive_language_usage_ledger` (non-authoritative quality adaptation signals) | `docs/DB_WIRING/PH1_LEARNING_ADAPTIVE.md` |
| PH1.CAPREQ | Capability request lifecycle tables | `docs/DB_WIRING/PH1_CAPREQ.md` |
| PH1.EXPLAIN | Planned; expected explanation mapping + audit-only writes scope (to be locked) | `docs/DB_WIRING/PH1_EXPLAIN.md` (stub) |
| PH1.REM | `reminders`, `reminder_occurrences`, `reminder_delivery_attempts` (timing mechanics truth; includes `BCAST_MHP_FOLLOWUP` scheduling handoff) | `docs/DB_WIRING/PH1_REM.md` |
| PH1.BCAST | `comms.broadcast_envelopes_ledger/current`, `comms.broadcast_recipients_current`, `comms.broadcast_delivery_attempts_ledger`, `comms.broadcast_ack_ledger` (broadcast lifecycle truth) | `docs/DB_WIRING/PH1_BCAST.md` |
| PH1.DELIVERY | `comms.delivery_attempts_ledger/current`, `comms.delivery_provider_health` (provider send/status attempt truth) | `docs/DB_WIRING/PH1_DELIVERY.md` |
| PH1.EMO | Planned; emotional profile/privacy directive tables scope (to be locked) | `docs/DB_WIRING/PH1_EMO.md` (stub) |

## Non-Duplication Rule

- Do not duplicate per-table column lists, FK lists, or acceptance-test details in this file.
- Those details live only in:
  - `docs/04_KERNEL_CONTRACTS.md`
  - `docs/DB_WIRING/*.md`
