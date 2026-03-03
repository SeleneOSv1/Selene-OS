# Idempotency Registry (Run 1 Foundation Lock)

`IDEMPOTENCY_KEYS.json` defines deterministic idempotency keys for PH1.J and PH1.F write paths.

## Required Write Paths
- `audit_append` (PH1.J)
- `evidence_persist` (PH1.F)
- `policy_snapshot_persist` (PH1.F)

## Hard Rules
- Key recipes are pure deterministic functions of canonical packet fields.
- No randomness, timestamps, or environment-specific data in key material.
- Duplicate behavior is declared per write path and must be enforced consistently.
