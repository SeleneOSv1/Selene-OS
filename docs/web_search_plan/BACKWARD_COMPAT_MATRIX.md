# Backward Compatibility Matrix (Run 1 Foundation Lock)

## Allowed (Backward-Compatible)
- Add optional fields.
- Tighten enum values only when backward acceptance is preserved for existing values.

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
