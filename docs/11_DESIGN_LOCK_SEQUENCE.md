# Selene OS Design Lock Sequence (Authoritative)

Purpose:
- Keep one fixed, ordered checklist for what must be finalized before broad runtime build-out.
- Ensure we always know "what is next" without restating the sequence each time.

Execution rule:
- Work top-to-bottom in strict order.
- Do not start broad runtime wiring until items 1-9 are locked.

Status legend:
- `OPEN` = not locked yet
- `LOCKED` = finalized and cross-doc aligned

## Ordered Checklist

| Order | Item | Status | Notes |
| --- | --- | --- | --- |
| 1 | Lock engine boundaries and ownership | LOCKED | Locked on 2026-02-11 across Constitution + Engine Map + Engine Registry + DB Ownership Matrix |
| 2 | Lock capability maps | LOCKED | Locked on 2026-02-11 across Kernel Contracts (KC.7A) + Constitution (ECM.*) + Engine Map + Engine Registry + Simulation Catalog + Blueprint Registry |
| 3 | Lock DB schemas per engine | LOCKED | Locked on 2026-02-11 across Kernel Contracts (KC.16..KC.24) + Constitution (F.4.*) + DB Ownership Matrix (table contract bindings) |
| 4 | Lock WorkOrder schema | LOCKED | Locked on 2026-02-11 across Kernel Contracts (KC.23), Constitution (F.4.18), DB Ownership Matrix (WorkOrder tables ACTIVE + bindings), Engine Map, Engine Registry |
| 5 | Lock Simulation catalog schema | LOCKED | Locked on 2026-02-11 across Kernel Contracts (KC.7), Constitution (Section 0.4 + SCS), Simulation Catalog (explicit defaults; no placeholders), Engine Map, Engine Registry |
| 6 | Lock Blueprint registry schema | LOCKED | Locked on 2026-02-11 across Kernel Contracts (KC.6), Constitution (Section 0.5 + PBS), Blueprint Registry (schema lock + completeness defaults), Engine Map, Engine Registry |
| 7 | Lock PH2.ACCESS.002 instance + override schema | LOCKED | Locked on 2026-02-11 across Kernel Contracts (KC.19), Constitution (F.4.15 + Section 0.6), DB Ownership Matrix (`access_instances`/`access_overrides` ownership), Engine Map, Engine Registry |
| 8 | Lock Audit event schema | LOCKED | Locked on 2026-02-11 across Kernel Contracts (KC.9 + KC.22.9), Constitution (Section 0.7 + J.4 + F.4.9), DB Ownership Matrix (`audit_events` linkage), Engine Map, Engine Registry |
| 9 | Lock idempotency + lease contracts | LOCKED | Locked on 2026-02-11 across Kernel Contracts (KC.10 + KC.13 + KC.23.4 + KC.23.5), Constitution (Section 0.8 + OS/LEASE rules), DB Ownership Matrix (lease/idempotency linkage), Engine Map, Engine Registry |
| 10 | Only then do minimal runtime wiring | LOCKED | Locked on 2026-02-11 with minimal contract-first runtime slices wired in `crates/selene_os` (LINK/ONB/WAKE/VOICE.ID/POSITION) and verified by `cargo test -p selene_os` |

## Working Rule For Every Session

- Determine the first row with `Status=OPEN`.
- Treat that row as the current "next item."
- After completion, update status to `LOCKED` and add a short note in `docs/03_BUILD_LEDGER.md`.

## Continuous Canon Update Rule (Mandatory)

After every engine or workflow change, update canonical docs in the same commit:
- `docs/07_ENGINE_REGISTRY.md` (engine row/link/state)
- `docs/10_DB_OWNERSHIP_MATRIX.md` (ownership summary/link scope)
- `docs/COVERAGE_MATRIX.md` (coverage status)
- `docs/08_SIMULATION_CATALOG.md` (if simulations changed)
- `docs/09_BLUEPRINT_REGISTRY.md` + `docs/BLUEPRINTS/*.md` (if blueprint mapping/records changed)
- `docs/11_DESIGN_LOCK_SEQUENCE.md` (if lock status changed)

Hard rule:
- No deferred doc updates.
- No duplicate inventories across summary docs.

## Next Phase After Items 1-10

- DB wiring phase is tracked in:
  - `docs/DB_WIRING/00_DB_WIRING_DESIGN_LOCK_SEQUENCE.md`
- Rule: complete DB wiring one engine at a time in that sequence and report the next engine automatically after each pass/block.
