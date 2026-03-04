# Production SLO Lock

Run 30 production lock fails closed unless all SLO gates below are green.

## Hard Thresholds

1. Citation coverage (answer cases): exactly `1.0`.
2. Refusal correctness (refusal cases): all refusal cases must pass.
3. Freshness TTL compliance (real-time mode): stale data must be refused.
4. Determinism replay: replay snapshot regression gate must pass.

## Enforcement

- `scripts/web_search_plan/check_quality_gates.sh` enforces citation coverage and refusal correctness.
- `scripts/web_search_plan/check_realtime_api_mode.sh` enforces stale refusal behavior for freshness.
- `scripts/web_search_plan/check_replay_harness.sh` enforces deterministic replay snapshots.

## Release Rule

If any SLO gate fails, release is blocked (`no-release-on-red`).
