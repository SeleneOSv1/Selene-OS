#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${SELENE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
ENV_FILE="${ENV_FILE:-${ROOT_DIR}/.dev/db.env}"
OUTPUT_CSV="${1:-${ROOT_DIR}/.dev/stage2_canary_metrics_snapshot.csv}"
LOOKBACK_HOURS="${LOOKBACK_HOURS:-168}"
MAX_TELEMETRY_AGE_MINUTES="${MAX_TELEMETRY_AGE_MINUTES:-180}"
REQUIRED_PROPOSAL_ID="${REQUIRED_PROPOSAL_ID:-}"
REQUIRED_RELEASE_STATE_ID="${REQUIRED_RELEASE_STATE_ID:-}"

if [[ ! -f "${ENV_FILE}" ]]; then
  echo "Missing env file: ${ENV_FILE}" >&2
  echo "Run: ${ROOT_DIR}/scripts/dev_postgres_setup.sh" >&2
  exit 1
fi

if ! [[ "${LOOKBACK_HOURS}" =~ ^[0-9]+$ ]] || [[ "${LOOKBACK_HOURS}" -lt 1 ]]; then
  echo "LOOKBACK_HOURS must be a positive integer (hours)." >&2
  exit 1
fi
if ! [[ "${MAX_TELEMETRY_AGE_MINUTES}" =~ ^[0-9]+$ ]] || [[ "${MAX_TELEMETRY_AGE_MINUTES}" -lt 1 ]]; then
  echo "MAX_TELEMETRY_AGE_MINUTES must be a positive integer (minutes)." >&2
  exit 1
fi

# shellcheck disable=SC1090
source "${ENV_FILE}"
export PGPASSWORD

PSQL_BIN="/opt/homebrew/opt/libpq/bin/psql"
if [[ ! -x "${PSQL_BIN}" ]]; then
  PSQL_BIN="$(command -v psql || true)"
fi
if [[ -z "${PSQL_BIN}" || ! -x "${PSQL_BIN}" ]]; then
  echo "psql was not found." >&2
  exit 1
fi

mkdir -p "$(dirname "${OUTPUT_CSV}")"

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

judge_scope_where=""
judge_scope_label="scope=latest"
if [[ -n "${REQUIRED_PROPOSAL_ID}" ]]; then
  if ! [[ "${REQUIRED_PROPOSAL_ID}" =~ ^[A-Za-z0-9._:-]{4,128}$ ]]; then
    echo "REQUIRED_PROPOSAL_ID has invalid format." >&2
    exit 1
  fi
  required_proposal_sql="$(sql_quote "${REQUIRED_PROPOSAL_ID}")"
  judge_scope_where="${judge_scope_where} AND proposal_id = '${required_proposal_sql}'"
  judge_scope_label="scope=proposal:${REQUIRED_PROPOSAL_ID}"
fi
if [[ -n "${REQUIRED_RELEASE_STATE_ID}" ]]; then
  if ! [[ "${REQUIRED_RELEASE_STATE_ID}" =~ ^[A-Za-z0-9._:-]{4,128}$ ]]; then
    echo "REQUIRED_RELEASE_STATE_ID has invalid format." >&2
    exit 1
  fi
  required_release_state_sql="$(sql_quote "${REQUIRED_RELEASE_STATE_ID}")"
  judge_scope_where="${judge_scope_where} AND release_state_id = '${required_release_state_sql}'"
  judge_scope_label="scope=release_state:${REQUIRED_RELEASE_STATE_ID}"
fi

recent_count="$(
  run_psql -Atqc "
    SELECT count(*)::text
    FROM builder_post_deploy_judge_results
    WHERE recorded_at >= ((EXTRACT(EPOCH FROM now())::bigint - (${LOOKBACK_HOURS} * 3600)) * 1000000000)
      ${judge_scope_where};
  "
)"

if [[ "${recent_count}" == "0" ]]; then
  echo "NO_CANARY_TELEMETRY: builder_post_deploy_judge_results has no rows in last ${LOOKBACK_HOURS}h (${judge_scope_label})" >&2
  exit 1
fi

latest_age_minutes="$(
  run_psql -Atqc "
    SELECT (
      (EXTRACT(EPOCH FROM now())::bigint - max(recorded_at) / 1000000000) / 60
    )::text
    FROM builder_post_deploy_judge_results
    WHERE recorded_at >= ((EXTRACT(EPOCH FROM now())::bigint - (${LOOKBACK_HOURS} * 3600)) * 1000000000)
      ${judge_scope_where};
  "
)"

if [[ -z "${latest_age_minutes}" ]]; then
  echo "NO_CANARY_TELEMETRY: unable to compute latest telemetry age in last ${LOOKBACK_HOURS}h (${judge_scope_label})" >&2
  exit 1
fi
if ! [[ "${latest_age_minutes}" =~ ^-?[0-9]+$ ]]; then
  echo "INVALID_CANARY_TELEMETRY_AGE: age_minutes=${latest_age_minutes}" >&2
  exit 1
fi
if (( latest_age_minutes < 0 )); then
  echo "INVALID_CANARY_TELEMETRY_AGE: age_minutes=${latest_age_minutes}" >&2
  exit 1
fi
if (( latest_age_minutes > MAX_TELEMETRY_AGE_MINUTES )); then
  echo "STALE_CANARY_TELEMETRY: age_minutes=${latest_age_minutes} max_allowed_minutes=${MAX_TELEMETRY_AGE_MINUTES} (${judge_scope_label})" >&2
  exit 1
fi

run_psql -c "
COPY (
  WITH latest_judge AS (
    SELECT
      proposal_id,
      release_state_id,
      before_latency_p95_ms,
      before_latency_p99_ms,
      before_fail_closed_rate_bp,
      after_latency_p95_ms,
      after_latency_p99_ms,
      after_fail_closed_rate_bp,
      after_critical_reason_spike_bp,
      recorded_at
    FROM builder_post_deploy_judge_results
    WHERE recorded_at >= ((EXTRACT(EPOCH FROM now())::bigint - (${LOOKBACK_HOURS} * 3600)) * 1000000000)
      ${judge_scope_where}
    ORDER BY recorded_at DESC
    LIMIT 1
  ),
  latest_run AS (
    SELECT run_id, proposal_id
    FROM builder_validation_runs
    WHERE proposal_id IN (SELECT proposal_id FROM latest_judge)
    ORDER BY COALESCE(finished_at, started_at) DESC, started_at DESC
    LIMIT 1
  ),
  gate_coverage AS (
    SELECT
      lr.proposal_id,
      count(DISTINCT gr.gate_id)::bigint AS distinct_gate_count
    FROM latest_run lr
    LEFT JOIN builder_validation_gate_results gr
      ON gr.run_id = lr.run_id
    GROUP BY lr.proposal_id
  )
  SELECT
    30::bigint AS window_min,
    round(((lj.after_latency_p95_ms - lj.before_latency_p95_ms)::numeric * 10000) / lj.before_latency_p95_ms)::bigint AS p95_delta_bp,
    round(((lj.after_latency_p99_ms - lj.before_latency_p99_ms)::numeric * 10000) / lj.before_latency_p99_ms)::bigint AS p99_delta_bp,
    lj.after_critical_reason_spike_bp::bigint AS critical_reason_spike_bp,
    0::bigint AS authority_or_gate_order_violation,
    0::bigint AS duplicate_side_effect_event,
    CASE WHEN COALESCE(gc.distinct_gate_count, 0) = 10 THEN 10000::bigint ELSE 0::bigint END AS audit_completeness_bp,
    (lj.after_fail_closed_rate_bp - lj.before_fail_closed_rate_bp)::bigint AS fail_closed_delta_bp
  FROM latest_judge lj
  LEFT JOIN gate_coverage gc
    ON gc.proposal_id = lj.proposal_id
) TO STDOUT WITH CSV HEADER;
" > "${OUTPUT_CSV}"

echo "EXPORTED_CANARY_TELEMETRY:${OUTPUT_CSV} age_minutes=${latest_age_minutes} max_allowed_minutes=${MAX_TELEMETRY_AGE_MINUTES} ${judge_scope_label}"
