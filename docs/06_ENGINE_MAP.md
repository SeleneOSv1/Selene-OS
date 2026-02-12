# Engine Map (Summary + Navigation)

Purpose:
- Provide a fast orientation view of runtime flow.
- Point to canonical docs for authoritative details.

This file is non-canonical by design.

## Canonical References

- Design truth and ownership rules: `docs/00_DESIGN_TRUTH_OPTION_B.md`
- Engine registry (authoritative engine list): `docs/07_ENGINE_REGISTRY.md`
- Simulation inventory (authoritative): `docs/08_SIMULATION_CATALOG.md`
- Blueprint registry (authoritative mapping): `docs/09_BLUEPRINT_REGISTRY.md`
- DB ownership summary (authoritative): `docs/10_DB_OWNERSHIP_MATRIX.md`
- Design lock status (authoritative): `docs/11_DESIGN_LOCK_SEQUENCE.md`
- Coverage/status matrix (authoritative): `docs/COVERAGE_MATRIX.md`
- Detailed engine contracts: `docs/DB_WIRING/*.md` and `docs/ECM/*.md`
- PH1.M vNext memory architecture: `docs/12_MEMORY_ARCHITECTURE.md`

## Phase A Navigation (Design Completion)

- `PH1.F`: `docs/DB_WIRING/PH1_F.md` + `docs/ECM/PH1_F.md`
- `PH1.J`: `docs/DB_WIRING/PH1_J.md` + `docs/ECM/PH1_J.md`
- `SELENE_OS_CORE_TABLES`: `docs/DB_WIRING/SELENE_OS_CORE_TABLES.md` + `docs/ECM/SELENE_OS_CORE_TABLES.md`
- Planned-but-not-yet-finalized engines are tracked only in `docs/07_ENGINE_REGISTRY.md` and `docs/COVERAGE_MATRIX.md`.

## Phase B Navigation (Identity/Access)

- `PH1.L`: `docs/DB_WIRING/PH1_L.md` + `docs/ECM/PH1_L.md`
- `PH1.VOICE.ID`: `docs/DB_WIRING/PH1_VOICE_ID.md` + `docs/ECM/PH1_VOICE_ID.md`
- `PH1.ACCESS.001_PH2.ACCESS.002`: `docs/DB_WIRING/PH1_ACCESS_001_PH2_ACCESS_002.md` + `docs/ECM/PH1_ACCESS_001_PH2_ACCESS_002.md`

## Phase C Navigation (Perception + Understanding + Orchestration)

- `PH1.K`: `docs/DB_WIRING/PH1_K.md` + `docs/ECM/PH1_K.md`
- `PH1.W`: `docs/DB_WIRING/PH1_W.md` + `docs/ECM/PH1_W.md`
- `PH1.C`: `docs/DB_WIRING/PH1_C.md` + `docs/ECM/PH1_C.md`
- `PH1.NLP`: `docs/DB_WIRING/PH1_NLP.md` + `docs/ECM/PH1_NLP.md`
- `PH1.D`: `docs/DB_WIRING/PH1_D.md` + `docs/ECM/PH1_D.md`
- `PH1.X`: `docs/DB_WIRING/PH1_X.md` + `docs/ECM/PH1_X.md`

## Phase F Navigation (Memory/Learning)

- `PH1.M (vNext++)`: `docs/DB_WIRING/PH1_M.md` + `docs/ECM/PH1_M.md` + `docs/12_MEMORY_ARCHITECTURE.md` (threads + graph + paging + tiered auto-resume + pending WorkOrder continuity + retention mode)
- Memory workflow blueprints:
  - `docs/BLUEPRINTS/MEMORY_QUERY.md`
  - `docs/BLUEPRINTS/MEMORY_FORGET_REQUEST.md`
  - `docs/BLUEPRINTS/MEMORY_REMEMBER_REQUEST.md`

## Runtime Flow (High-Level)

```text
Voice: PH1.K -> PH1.W -> PH1.VOICE.ID -> PH1.C -> PH1.NLP -> PH1.X -> PH1.WRITE -> PH1.TTS
Text:  UI -> transcript_ok-equivalent -> PH1.NLP -> PH1.X -> PH1.WRITE -> UI
```

Execution law:
- Engines never call engines directly.
- Selene OS orchestrates all cross-engine sequencing.
- Side effects require Access + Simulation (`No Simulation -> No Execution`).

## Turn Wiring Graph (Authoritative)

Always-on turn (voice):

```text
PH1.K -> PH1.W -> PH1.VOICE.ID -> PH1.C -> PH1.SRL -> (PH1.PUZZLE optional) -> PH1.NLP -> PH1.X
```

Assists (called by Selene OS, never engine-to-engine):
- PH1.ENDPOINT assists capture boundaries (inputs from PH1.K/PH1.C; outputs endpoint hints to Selene OS, which passes them to PH1.C/PH1.K).
- PH1.LANG assists PH1.C/PH1.SRL/PH1.NLP.
- PH1.ATTN assists PH1.NLP/PH1.CONTEXT.
- PH1.PRUNE assists PH1.X when multiple missing fields exist.
- PH1.DIAG runs before PH1.X finalizes a move.
- PH1.EXPLAIN runs only when the user asks “why?”
- PH1.SEARCH/PH1.WEBINT/PH1.PREFETCH assist planning and evidence interpretation; PH1.E executes tools only.
- PH1.DOC/PH1.VISION are invoked only when user documents/images are provided; outputs are evidence bundles for PH1.CONTEXT/PH1.NLP.

Learning wiring (not in-turn execution path):
- PH1.LISTEN/PH1.PAE/PH1.CACHE/PH1.MULTI/PH1.CONTEXT feed hints and policy snapshots only (no execution path).
- PH1.PATTERN/PH1.RLL produce OFFLINE artifact proposals only.

Wiring class declaration:
- ALWAYS_ON: `PH1.K`, `PH1.W`, `PH1.VOICE.ID`, `PH1.C`, `PH1.SRL`, `PH1.NLP`, `PH1.X`, `PH1.CONTEXT`
- TURN_OPTIONAL: `PH1.PUZZLE`, `PH1.ENDPOINT`, `PH1.LANG`, `PH1.ATTN`, `PH1.DOC`, `PH1.VISION`, `PH1.PRUNE`, `PH1.DIAG`, `PH1.SEARCH`, `PH1.WEBINT`, `PH1.PREFETCH`, `PH1.EXPLAIN`, `PH1.LISTEN`, `PH1.PAE`, `PH1.CACHE`, `PH1.MULTI`, `PH1.KG`
- OFFLINE_ONLY: `PH1.PATTERN`, `PH1.RLL`

## Design Hygiene

- Do not place simulation inventories in this file.
- Do not place blueprint records in this file.
- Do not place lock status tables in this file.
- Keep this document short; link out to canonical sources above.
