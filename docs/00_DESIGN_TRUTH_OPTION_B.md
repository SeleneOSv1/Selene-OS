# Option B Design Truth (Canonical)

This file defines where design truth lives and prevents duplicate registries.

## Canonical Sources (Do Not Duplicate)

Control-plane canonical docs:
1. `docs/07_ENGINE_REGISTRY.md` (engine list + engine status links)
2. `docs/08_SIMULATION_CATALOG.md` (simulation inventory only)
3. `docs/09_BLUEPRINT_REGISTRY.md` (intent -> process mapping only)
4. `docs/BLUEPRINTS/*.md` (full blueprint records only)
5. `docs/10_DB_OWNERSHIP_MATRIX.md` (DB ownership summary only)
6. `docs/11_DESIGN_LOCK_SEQUENCE.md` (design lock status only)
7. `docs/COVERAGE_MATRIX.md` (cross-link completion matrix only)
8. `docs/12_MEMORY_ARCHITECTURE.md` (canonical PH1.M architecture contract)

Detail canonical docs:
1. `docs/DB_WIRING/*.md` (engine DB wiring contracts)
2. `docs/ECM/*.md` (engine capability maps)

## Non-Canonical (Summary / Index / Evidence)

- `docs/05_OS_CONSTITUTION.md` is summary/governance text and is not auto-updating inventory truth.
- `docs/06_ENGINE_MAP.md` is summary + navigation only.
- `docs/02_BUILD_PLAN.md` is roadmap only.
- `docs/03_BUILD_LEDGER.md` is append-only evidence/history only.
- `docs/00_INDEX.md` is navigation only.

## Update Rule (Required in Same Change)

When design changes touch an engine or workflow, update canonical docs in the same commit:

1. `docs/07_ENGINE_REGISTRY.md` (engine row/link/status)
2. `docs/10_DB_OWNERSHIP_MATRIX.md` (ownership links/rows as needed)
3. `docs/COVERAGE_MATRIX.md` (coverage state)
4. `docs/08_SIMULATION_CATALOG.md` (if simulations changed)
5. `docs/09_BLUEPRINT_REGISTRY.md` + `docs/BLUEPRINTS/*.md` (if blueprint mapping/records changed)
6. `docs/11_DESIGN_LOCK_SEQUENCE.md` (if lock status changed)
7. `docs/DB_WIRING/*.md` and/or `docs/ECM/*.md` (if engine contracts changed)

## No-Duplicate-Inventory Rule

- Simulation inventory exists only in `docs/08_SIMULATION_CATALOG.md`.
- Blueprint inventory mapping exists only in `docs/09_BLUEPRINT_REGISTRY.md`.
- Full blueprint records exist only in `docs/BLUEPRINTS/*.md`.
- Engine status tracking exists only in `docs/COVERAGE_MATRIX.md` (with links from `docs/07_ENGINE_REGISTRY.md`).
- Lock status exists only in `docs/11_DESIGN_LOCK_SEQUENCE.md`.

## Scope Rule

Current active engine scope follows `docs/DB_WIRING/00_DB_WIRING_DESIGN_LOCK_SEQUENCE.md`.
Memory command workflows are canonical process records in:
- `docs/BLUEPRINTS/MEMORY_QUERY.md`
- `docs/BLUEPRINTS/MEMORY_FORGET_REQUEST.md`
- `docs/BLUEPRINTS/MEMORY_REMEMBER_REQUEST.md`

Phase C design contracts are canonical in:
- `docs/DB_WIRING/PH1_K.md`, `docs/DB_WIRING/PH1_W.md`, `docs/DB_WIRING/PH1_C.md`, `docs/DB_WIRING/PH1_NLP.md`, `docs/DB_WIRING/PH1_D.md`, `docs/DB_WIRING/PH1_X.md`
- `docs/ECM/PH1_K.md`, `docs/ECM/PH1_W.md`, `docs/ECM/PH1_C.md`, `docs/ECM/PH1_NLP.md`, `docs/ECM/PH1_D.md`, `docs/ECM/PH1_X.md`
If a required contract is missing, mark `BLOCKER` in the engine detail doc and in `docs/COVERAGE_MATRIX.md`, then stop.

## DOC INVARIANTS (Non-Negotiable)

1. One canonical source per topic:
- engine list: `docs/07_ENGINE_REGISTRY.md`
- simulation inventory: `docs/08_SIMULATION_CATALOG.md`
- blueprint mapping: `docs/09_BLUEPRINT_REGISTRY.md`
- blueprint records: `docs/BLUEPRINTS/*.md`
- DB ownership summary: `docs/10_DB_OWNERSHIP_MATRIX.md`
- lock status: `docs/11_DESIGN_LOCK_SEQUENCE.md`
- coverage status: `docs/COVERAGE_MATRIX.md`

2. Summary docs must not carry duplicate inventories:
- `docs/06_ENGINE_MAP.md`, `docs/05_OS_CONSTITUTION.md`, `docs/00_INDEX.md`, `docs/02_BUILD_PLAN.md` are summary/navigation/roadmap only.

3. Detailed implementation contracts live only in:
- `docs/DB_WIRING/*.md`
- `docs/ECM/*.md`
- `docs/04_KERNEL_CONTRACTS.md`

4. Historical evidence remains append-only in:
- `docs/03_BUILD_LEDGER.md`
