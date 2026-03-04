# Rollback Plan: Web Search v1.0.0

## 1) Disable Go-Live Flag

Set the runtime flag to disabled in production:

- `SELENE_WEB_SEARCH_ENABLED=false`

Redeploy configuration and restart the service fleet.

## 2) Roll Back to Previous Tag

Use git tags for deterministic rollback:

1. `git fetch --tags`
2. Identify previous stable tag before `web-search-v1.0.0`.
3. Deploy that prior tag in production.

## 3) Verify Disabled State

1. Confirm runtime environment has `SELENE_WEB_SEARCH_ENABLED=false`.
2. Confirm web-search lane is not dispatching in production telemetry.
3. Confirm no web-search release evidence is generated after disable.

## 4) Re-Run Release Lock Scripts

Run the following in the target rollback candidate environment:

- `scripts/web_search_plan/check_release_lock.sh`
- `scripts/web_search_plan/check_trace_matrix.sh`
- `scripts/web_search_plan/check_slo_lock.sh`
- `scripts/web_search_plan/generate_release_evidence_pack.sh`

All must pass before re-enabling go-live.
