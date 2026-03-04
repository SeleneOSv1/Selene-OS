# Web Search Go-Live (v1.0.0)

This runbook defines the production enable switch for the web-search lane without storing secrets in git.

## Required Environment Variables (Names Only)

- `SELENE_WEB_SEARCH_ENABLED`
- `BRAVE_API_KEY`
- `OPENAI_API_KEY`
- `GOOGLE_STT_CREDENTIALS`
- `HTTP_PROXY` (optional)
- `HTTPS_PROXY` (optional)

## Enable Flag

- Enable: `SELENE_WEB_SEARCH_ENABLED=true`
- Disable: `SELENE_WEB_SEARCH_ENABLED=false`

## Production Enable Procedure

1. Set required environment variables in the production secret manager and runtime environment.
2. Set `SELENE_WEB_SEARCH_ENABLED=true` in the production deployment environment.
3. Deploy the tagged release `web-search-v1.0.0`.
4. Verify release gates using:
   - `scripts/web_search_plan/check_release_lock.sh`
   - `scripts/web_search_plan/check_trace_matrix.sh`
   - `scripts/web_search_plan/check_slo_lock.sh`
5. Confirm latest release evidence pack is archived under `docs/web_search_plan/release_archive/web-search-v1.0.0/`.

## Rollback

Follow `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/web_search_plan/ROLLBACK_WEB_SEARCH_V1.md`.
