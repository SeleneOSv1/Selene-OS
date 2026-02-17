#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

ENV_FILE="${ENV_FILE:-.dev/db.env}"
PROPOSAL_ID="${PROPOSAL_ID:-}"
LOOKBACK_HOURS="${LOOKBACK_HOURS:-168}"
MAX_TELEMETRY_AGE_MINUTES="${MAX_TELEMETRY_AGE_MINUTES:-180}"

fail() {
  echo "PRODUCTION_SOAK_FAIL:$1" >&2
  exit 1
}

if [[ ! -f "${ENV_FILE}" ]]; then
  fail "missing_env_file path=${ENV_FILE}"
fi
if ! [[ "${LOOKBACK_HOURS}" =~ ^[0-9]+$ ]] || [[ "${LOOKBACK_HOURS}" -lt 1 ]]; then
  fail "invalid_LOOKBACK_HOURS expected_positive_integer actual=${LOOKBACK_HOURS}"
fi
if ! [[ "${MAX_TELEMETRY_AGE_MINUTES}" =~ ^[0-9]+$ ]] || [[ "${MAX_TELEMETRY_AGE_MINUTES}" -lt 1 ]]; then
  fail "invalid_MAX_TELEMETRY_AGE_MINUTES expected_positive_integer actual=${MAX_TELEMETRY_AGE_MINUTES}"
fi

# shellcheck disable=SC1090
source "${ENV_FILE}"
export PGPASSWORD

PSQL_BIN="/opt/homebrew/opt/libpq/bin/psql"
if [[ ! -x "${PSQL_BIN}" ]]; then
  PSQL_BIN="$(command -v psql || true)"
fi
if [[ -z "${PSQL_BIN}" || ! -x "${PSQL_BIN}" ]]; then
  fail "psql_not_found"
fi

run_psql() {
  "${PSQL_BIN}" \
    -h "${PGHOST}" \
    -p "${PGPORT}" \
    -U "${PGUSER}" \
    -d "${PGDATABASE}" \
    -v ON_ERROR_STOP=1 \
    "$@"
}

sql_quote() {
  printf "%s" "$1" | sed "s/'/''/g"
}

if [[ -z "${PROPOSAL_ID}" ]]; then
  PROPOSAL_ID="$(
    run_psql -Atqc "
      SELECT proposal_id
      FROM builder_release_states
      ORDER BY recorded_at DESC, release_row_id DESC
      LIMIT 1;
    "
  )"
fi
if [[ -z "${PROPOSAL_ID}" ]]; then
  fail "proposal_not_found"
fi

proposal_sql="$(sql_quote "${PROPOSAL_ID}")"
latest_release_row="$(
  run_psql -AtF '|' -c "
    SELECT release_state_id, stage, status
    FROM builder_release_states
    WHERE proposal_id = '${proposal_sql}'
    ORDER BY recorded_at DESC, release_row_id DESC
    LIMIT 1;
  "
)"
if [[ -z "${latest_release_row}" ]]; then
  fail "release_state_not_found proposal_id=${PROPOSAL_ID}"
fi

IFS='|' read -r release_state_id stage status <<< "${latest_release_row}"
if [[ "${stage}" != "PRODUCTION" || "${status}" != "COMPLETED" ]]; then
  fail "proposal_not_in_completed_production proposal_id=${PROPOSAL_ID} stage=${stage} status=${status}"
fi

approval_status="$(
  run_psql -Atqc "
    SELECT status
    FROM builder_approval_states
    WHERE proposal_id = '${proposal_sql}'
    ORDER BY recorded_at DESC, approval_row_id DESC
    LIMIT 1;
  "
)"
if [[ "${approval_status}" != "APPROVED" ]]; then
  fail "approval_status_not_approved proposal_id=${PROPOSAL_ID} approval_status=${approval_status:-NONE}"
fi

judge_row="$(
  run_psql -AtF '|' -c "
    SELECT judge_result_id, action, reason_code
    FROM builder_post_deploy_judge_results
    WHERE proposal_id = '${proposal_sql}'
      AND release_state_id = '$(sql_quote "${release_state_id}")'
    ORDER BY recorded_at DESC, judge_row_id DESC
    LIMIT 1;
  "
)"
if [[ -z "${judge_row}" ]]; then
  fail "missing_production_judge_result proposal_id=${PROPOSAL_ID} release_state_id=${release_state_id}"
fi
IFS='|' read -r judge_result_id judge_action judge_reason_code <<< "${judge_row}"
if [[ "${judge_action}" != "ACCEPT" ]]; then
  fail "production_judge_not_accept proposal_id=${PROPOSAL_ID} release_state_id=${release_state_id} action=${judge_action} reason_code=${judge_reason_code}"
fi

soak_metrics_csv="$(mktemp "${TMPDIR:-/tmp}/builder_production_soak_metrics.XXXXXX.csv")"
cleanup_soak_metrics() {
  rm -f "${soak_metrics_csv}"
}
trap cleanup_soak_metrics EXIT

REQUIRED_PROPOSAL_ID="${PROPOSAL_ID}" \
REQUIRED_RELEASE_STATE_ID="${release_state_id}" \
LOOKBACK_HOURS="${LOOKBACK_HOURS}" \
MAX_TELEMETRY_AGE_MINUTES="${MAX_TELEMETRY_AGE_MINUTES}" \
bash scripts/export_builder_stage2_canary_metrics.sh "${soak_metrics_csv}" >/dev/null

bash scripts/check_builder_stage2_promotion_gate.sh "${soak_metrics_csv}" >/dev/null

age_minutes="$(
  run_psql -Atqc "
    SELECT (
      (EXTRACT(EPOCH FROM now())::bigint - max(recorded_at) / 1000000000) / 60
    )::text
    FROM builder_post_deploy_judge_results
    WHERE proposal_id = '${proposal_sql}'
      AND release_state_id = '$(sql_quote "${release_state_id}")';
  "
)"
if [[ -z "${age_minutes}" ]]; then
  fail "production_judge_age_unavailable proposal_id=${PROPOSAL_ID} release_state_id=${release_state_id}"
fi

echo "CHECK_OK builder_production_soak_watchdog=pass proposal_id=${PROPOSAL_ID} release_state_id=${release_state_id} judge_result_id=${judge_result_id} age_minutes=${age_minutes} max_allowed_minutes=${MAX_TELEMETRY_AGE_MINUTES}"
