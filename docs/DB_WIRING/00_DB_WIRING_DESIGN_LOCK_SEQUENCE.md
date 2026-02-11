# Selene DB Wiring Design Lock Sequence (Authoritative)

Purpose:
- Track engine-by-engine DB wiring completion so execution order is deterministic and visible.
- Enforce "one engine at a time" with explicit pass/block status.

Hard execution rules:
- Do not skip order.
- Do not jump ahead.
- Do not invent missing schema.
- If a dependency/schema/contract is missing, mark `BLOCKED` and stop.

Status legend:
- `OPEN` = not started
- `IN_PROGRESS` = active engine slice
- `PASS` = DB-wired spec + repo + tests complete
- `BLOCKED` = cannot continue without dependency/schema fix

## Required Deliverables Per Engine

Every engine slice must include:
- `docs/DB_WIRING/PH1_<ENGINE>.md` (full DB wiring spec template)
- migrations/indices/FKs changes (only if needed)
- typed repository interfaces (read/write)
- DB wiring tests:
  - `AT-<ENGINE>-DB-01` tenant isolation enforced
  - `AT-<ENGINE>-DB-02` append-only enforced (for ledgers)
  - `AT-<ENGINE>-DB-03` idempotency dedupe works
  - `AT-<ENGINE>-DB-04` rebuild current from ledger (if current table exists)

## Engine Processing Order (Non-Negotiable)

| Order | Phase | Engine / Scope | Status | Notes |
| --- | --- | --- | --- | --- |
| 1 | A Foundations | PH1.F — Persistence Foundation | PASS | Spec + typed repo + DB wiring tests passed (`cargo test -p selene_storage`) |
| 2 | A Foundations | PH1.J — Audit Engine | PASS | Scoped audit envelope + scoped idempotency dedupe locked; DB wiring tests pass |
| 3 | A Foundations | Selene OS core tables (`work_orders`, `work_order_ledger`, `conversation_ledger`, `sessions`, `identities`, `devices`) | PASS | WorkOrder contracts + storage tables + typed repo + DB wiring tests completed |
| 4 | A Foundations | PBS tables (`blueprint_registry`, `process_blueprints`) | PASS | PBS contracts + storage tables/repo wiring + DB wiring tests completed |
| 5 | A Foundations | Simulation Catalog tables (`simulation_catalog`) | PASS | Simulation catalog contracts + storage tables/repo wiring + DB wiring tests completed |
| 6 | A Foundations | Engine Capability Maps tables (`engine_capability_maps`) | PASS | Engine capability map contracts + storage tables/repo wiring + DB wiring tests completed |
| 7 | A Foundations | `artifacts_ledger` (+ `tool_cache` if in scope) | PASS | Artifact ledger + tool cache contracts + storage tables/repo wiring + DB wiring tests completed |
| 8 | B Identity/Session/Access | PH1.L | PASS | Session lifecycle DB wiring locked on `sessions` + PH1.L typed repo/tests + session read indexes migration completed |
| 9 | B Identity/Session/Access | PH1.VOICE.ID | PASS | Voice enrollment DB wiring locked: typed repo + append-only guard + migration + DB wiring tests completed |
| 10 | B Identity/Session/Access | PH1.ACCESS.001 + PH2.ACCESS.002 | PASS | Access instance + override DB wiring locked: migration + typed repo + gate read-only path + DB wiring tests completed |
| 11 | C Perception/Understanding/Orchestration | PH1.K | PASS | Audio runtime event ledger + rebuildable current projection wired: migration + typed repo + DB tests completed |
| 12 | C Perception/Understanding/Orchestration | PH1.W | PASS | Wake enrollment/runtime tables + profile bindings wired: migration + typed repo + DB tests completed |
| 13 | C Perception/Understanding/Orchestration | PH1.C | PASS | Transcript gate DB wiring locked on existing `conversation_ledger` + `audit_events`; typed repo + DB tests completed |
| 14 | C Perception/Understanding/Orchestration | PH1.NLP | PASS | NLP decision DB wiring locked on existing `audit_events`; typed repo + DB tests completed |
| 15 | C Perception/Understanding/Orchestration | PH1.D | PASS | LLM router decision DB wiring locked on existing `audit_events`; typed repo + DB tests completed |
| 16 | C Perception/Understanding/Orchestration | PH1.X | PASS | Conversation directive DB wiring locked on existing `audit_events`; typed repo + DB tests completed |
| 17 | D Output | PH1.WRITE | PASS | Writing/formatting DB wiring locked on existing `audit_events`; typed repo + DB tests completed |
| 18 | D Output | PH1.TTS | PASS | TTS runtime DB wiring locked on existing `audit_events`; typed repo + DB tests completed |
| 19 | E Tooling + Onboarding | PH1.E | PASS | Tool router DB wiring locked on existing `audit_events`; typed repo + DB tests completed |
| 20 | E Tooling + Onboarding | PH1.LINK | PASS | Link lifecycle DB wiring locked: draft/token schema migration + typed repo + DB tests completed |
| 21 | E Tooling + Onboarding | PH1.ONB.CORE / PH1.ONB.ORCH / PH1.ONB.BIZ | PASS | Onboarding session DB wiring locked on current PH1.F onboarding runtime state + tenant scope/idempotency guards; typed repo + DB tests completed |
| 22 | E Tooling + Onboarding | PH1.POSITION | PASS | Position DB wiring locked on tenant-scoped current + append-only lifecycle ledger; typed repo + DB tests completed |
| 23 | F Memory + Learning | PH1.M | PASS | Memory DB wiring locked on existing `memory_ledger` + rebuildable `memory_current`; typed repo + DB tests completed |
| 24 | F Memory + Learning | PH1.PERSONA | PASS | Persona DB wiring locked on existing `audit_events` with tenant/device scope guards; typed repo + DB tests completed |
| 25 | F Memory + Learning | PH1.LEARN / PH1.FEEDBACK / PH1.KNOW (if in MVP scope) | PASS | Audit-backed feedback events + artifact-ledger learning/dictionary packs wired; typed repo + DB tests completed |
| 26 | G Capability Requests | PH1.CAPREQ | PASS | CAPREQ reopened as active slice: kernel/storage contracts + migration + typed repo + DB wiring tests completed (`AT-CAPREQ-DB-01..04`) |

## Next Engine Rule

- "Next engine" is always the first row with status `OPEN` after the last `PASS`.
- If current row is `BLOCKED`, next engine remains unchanged until blocker is cleared.

Current next engine:
- none (`PASS` for all rows in current sequence scope)
