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

## Runtime Flow (High-Level)

```text
Voice: PH1.K -> PH1.W -> PH1.VOICE.ID -> PH1.C -> PH1.NLP -> PH1.X -> PH1.WRITE -> PH1.TTS
Text:  UI -> transcript_ok-equivalent -> PH1.NLP -> PH1.X -> PH1.WRITE -> UI
```

Execution law:
- Engines never call engines directly.
- Selene OS orchestrates all cross-engine sequencing.
- Side effects require Access + Simulation (`No Simulation -> No Execution`).

## Design Hygiene

- Do not place simulation inventories in this file.
- Do not place blueprint records in this file.
- Do not place lock status tables in this file.
- Keep this document short; link out to canonical sources above.
