# Ownership Matrix (Run 1 Foundation Lock)

Machine-readable source of truth: `OWNERSHIP_MATRIX.json`.

Each entry defines:
- engine_id
- authority (`authoritative` | `non_authoritative`)
- allowed_actions
- must_not_do

Hard rule: authority boundaries are strict and fail-closed.
