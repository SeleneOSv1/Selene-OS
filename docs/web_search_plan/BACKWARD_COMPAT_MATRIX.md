# Backward Compatibility Matrix (Run 1 Foundation Lock)

## Allowed (Backward-Compatible)
- Add optional fields.
- Tighten enum values only when backward acceptance is preserved for existing values.
- Add new packet definitions with explicit `consumer_expected_version`.

## Blocked (Backward-Incompatible)
- Rename fields.
- Remove fields.
- Change field type.
- Change field semantics.
- Reorder meaning-dependent arrays without explicit ordering contract.

## Required for Blocked Change
1. Bump `schema_version`.
2. Update this matrix.
3. Refresh valid/invalid fixtures.
4. Update `CONTRACT_HASH_MANIFEST.json`.

## Recorded Blocked Change (Phase A Dedup/Contract Fix)
- `ComparisonPacket` was split into two explicit packet contracts:
  - `TemporalComparisonPacket`
  - `CompetitiveComparisonPacket`
- This removes ambiguous dual-shape semantics and is treated as a blocked contract change with refreshed fixtures and validators.
