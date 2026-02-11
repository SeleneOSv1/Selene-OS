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
If a required contract is missing, mark `BLOCKER` in the engine detail doc and in `docs/COVERAGE_MATRIX.md`, then stop.
