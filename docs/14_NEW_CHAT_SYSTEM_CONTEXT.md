# Selene OS New Chat System Context (Pointer)

Last updated: 2026-02-28
Status: non-canonical pointer doc (kept for compatibility)

## Purpose

This file is a lightweight handoff pointer for fresh chats.
Authoritative design/runtime truth must come from canonical docs, not from this file.

## Use This First

- Primary context pack: `docs/15_FULL_SYSTEM_BUILD_CONTEXT.md`
- Canonical truth model: `docs/00_DESIGN_TRUTH_OPTION_B.md`

## Canonical Control Docs

- Engine inventory: `docs/07_ENGINE_REGISTRY.md`
- Simulation inventory: `docs/08_SIMULATION_CATALOG.md`
- Blueprint registry: `docs/09_BLUEPRINT_REGISTRY.md`
- DB ownership: `docs/10_DB_OWNERSHIP_MATRIX.md`
- Design lock sequence: `docs/11_DESIGN_LOCK_SEQUENCE.md`
- Coverage status: `docs/COVERAGE_MATRIX.md`
- Runtime law pointers: `docs/05_OS_CONSTITUTION.md`

## Readiness / Verification Commands

```bash
git branch --show-current
git rev-parse HEAD
git status --short

bash scripts/check_ph1_readiness_strict.sh
bash scripts/check_engine_tracker_duplicates.sh
```

## Rule

If this file conflicts with any canonical doc above, canonical docs win.
