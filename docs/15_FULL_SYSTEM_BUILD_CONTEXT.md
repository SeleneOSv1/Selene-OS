# Selene OS Full System Build Context (Living Navigation)

Last updated: 2026-02-28
Status: non-canonical context/navigation

## 1) What This File Is

This file is a living orientation guide for contributors.
It must not be treated as authoritative contract, inventory, or policy truth.

## 2) Canonical Design Truth Model

Primary source:
- `docs/00_DESIGN_TRUTH_OPTION_B.md`

Canonical control docs:
- `docs/07_ENGINE_REGISTRY.md`
- `docs/08_SIMULATION_CATALOG.md`
- `docs/09_BLUEPRINT_REGISTRY.md`
- `docs/BLUEPRINTS/*.md`
- `docs/10_DB_OWNERSHIP_MATRIX.md`
- `docs/11_DESIGN_LOCK_SEQUENCE.md`
- `docs/COVERAGE_MATRIX.md`
- `docs/12_MEMORY_ARCHITECTURE.md`
- `docs/04_KERNEL_CONTRACTS.md`
- `docs/DB_WIRING/*.md`
- `docs/ECM/*.md`

## 3) Runtime/Policy Reference Pointers

- High-level law + pointer map: `docs/05_OS_CONSTITUTION.md`
- Engine map pointer index (non-canonical): `docs/06_ENGINE_MAP.md`
- Active execution plan: `docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md`
- Engine review tracker: `docs/33_ENGINE_REVIEW_TRACKER.md`

## 4) Repo Topology (Stable)

- Workspace manifest: `Cargo.toml`
- Runtime crates: `crates/`
- Docs: `docs/`
- Guardrail/readiness scripts: `scripts/`

## 5) Operating Workflow (Current)

Start of session:

```bash
git status --short
git rev-parse --short HEAD
```

Design/runtime readiness proof:

```bash
bash scripts/check_ph1_readiness_strict.sh
bash scripts/check_engine_tracker_duplicates.sh
```

Targeted parity checks (as needed):

```bash
bash scripts/check_runtime_boundary_guards.sh
bash scripts/check_delivery_ownership_boundaries.sh
bash scripts/check_understanding_clarify_precedence.sh
bash scripts/check_learning_ownership_boundaries.sh
```

## 6) Guardrails

- Engines never call engines directly; Selene OS orchestrates.
- No Simulation -> No Execution.
- Use canonical docs for truth decisions.
- Keep `docs/03_BUILD_LEDGER.md` append-only for proof history.

## 7) Historical Snapshot Notes

Older chat/session snapshots and archived planning docs are historical evidence only.
Do not use archive docs as runtime or governance truth.
