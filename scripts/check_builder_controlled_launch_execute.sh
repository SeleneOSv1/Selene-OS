#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

ENV_FILE="${ENV_FILE:-.dev/db.env}"
PERMISSION_ENV_FILE="${PERMISSION_ENV_FILE:-.dev/builder_permission.env}"
PROPOSAL_ID="${PROPOSAL_ID:-}"
TARGET_STAGE="${TARGET_STAGE:-NEXT}"   # NEXT | CANARY | RAMP_25 | RAMP_50 | PRODUCTION
EXECUTE="${EXECUTE:-0}"                # 0 preview-only, 1 perform DB write
PRECHECK="${PRECHECK:-1}"              # 1 run pre-launch bundle before proceeding
REQUIRE_STAGE_JUDGE="${REQUIRE_STAGE_JUDGE:-1}"   # 1 require stage-bound post-deploy judge pass before advance
LAUNCH_EXECUTE_ACK="${LAUNCH_EXECUTE_ACK:-NO}"
LAUNCH_EXECUTE_IDEMPOTENCY_KEY="${LAUNCH_EXECUTE_IDEMPOTENCY_KEY:-}"

fail() {
  echo "CONTROLLED_LAUNCH_FAIL:$1" >&2
  exit 1
}

if [[ "${EXECUTE}" != "0" && "${EXECUTE}" != "1" ]]; then
  fail "invalid_EXECUTE expected=0_or_1 actual=${EXECUTE}"
fi
if [[ "${PRECHECK}" != "0" && "${PRECHECK}" != "1" ]]; then
  fail "invalid_PRECHECK expected=0_or_1 actual=${PRECHECK}"
fi
if [[ "${REQUIRE_STAGE_JUDGE}" != "0" && "${REQUIRE_STAGE_JUDGE}" != "1" ]]; then
  fail "invalid_REQUIRE_STAGE_JUDGE expected=0_or_1 actual=${REQUIRE_STAGE_JUDGE}"
fi
case "${TARGET_STAGE}" in
  NEXT|CANARY|RAMP_25|RAMP_50|PRODUCTION) ;;
  *) fail "invalid_TARGET_STAGE expected=NEXT|CANARY|RAMP_25|RAMP_50|PRODUCTION actual=${TARGET_STAGE}" ;;
esac

if [[ ! -f "${ENV_FILE}" ]]; then
  fail "missing_env_file path=${ENV_FILE}"
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

if [[ "${PRECHECK}" == "1" ]]; then
  bash scripts/check_builder_prelaunch_bundle.sh
fi

ENV_FILE="${PERMISSION_ENV_FILE}" bash scripts/check_builder_human_permission_gate.sh launch >/dev/null

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
current_release_row="$(
  run_psql -AtF '|' -c "
    SELECT release_state_id, stage, status, rollback_hook, rollback_ready
    FROM builder_release_states
    WHERE proposal_id = '${proposal_sql}'
    ORDER BY recorded_at DESC, release_row_id DESC
    LIMIT 1;
  "
)"
if [[ -z "${current_release_row}" ]]; then
  fail "release_state_not_found proposal_id=${PROPOSAL_ID}"
fi

IFS='|' read -r current_release_state_id current_stage current_status current_rollback_hook current_rollback_ready <<< "${current_release_row}"

case "${current_stage}" in
  PRODUCTION)
    if [[ "${current_status}" == "COMPLETED" ]]; then
      fail "no_next_stage current_stage=PRODUCTION status=COMPLETED"
    fi
    fail "terminal_stage_status_mismatch expected_status=COMPLETED actual=${current_status}"
    ;;
  ROLLED_BACK)
    if [[ "${current_status}" == "REVERTED" ]]; then
      fail "no_next_stage current_stage=ROLLED_BACK status=REVERTED"
    fi
    fail "terminal_stage_status_mismatch expected_status=REVERTED actual=${current_status}"
    ;;
  STAGING|CANARY|RAMP_25|RAMP_50) ;;
  *) fail "unknown_current_stage value=${current_stage}" ;;
esac

if [[ "${current_status}" != "ACTIVE" ]]; then
  fail "current_release_state_not_active proposal_id=${PROPOSAL_ID} stage=${current_stage} status=${current_status}"
fi

case "${current_stage}" in
  STAGING) computed_next_stage="CANARY" ;;
  CANARY) computed_next_stage="RAMP_25" ;;
  RAMP_25) computed_next_stage="RAMP_50" ;;
  RAMP_50) computed_next_stage="PRODUCTION" ;;
  *) fail "unknown_current_stage value=${current_stage}" ;;
esac
if [[ -z "${computed_next_stage}" ]]; then
  fail "no_next_stage current_stage=${current_stage}"
fi

if [[ "${REQUIRE_STAGE_JUDGE}" == "1" ]]; then
  case "${current_stage}" in
    CANARY|RAMP_25|RAMP_50)
      stage_bound_metrics_csv="$(mktemp "${TMPDIR:-/tmp}/builder_stage_bound_metrics.$$.XXXXXXXXXXXX.csv")"
      cleanup_stage_metrics() {
        rm -f "${stage_bound_metrics_csv}"
      }
      trap cleanup_stage_metrics EXIT
      REQUIRED_PROPOSAL_ID="${PROPOSAL_ID}" \
      REQUIRED_RELEASE_STATE_ID="${current_release_state_id}" \
      bash scripts/export_builder_stage2_canary_metrics.sh "${stage_bound_metrics_csv}" >/dev/null
      bash scripts/check_builder_stage2_promotion_gate.sh "${stage_bound_metrics_csv}" >/dev/null
      ;;
    STAGING) ;;
    *)
      fail "stage_judge_policy_unknown_stage stage=${current_stage}"
      ;;
  esac
fi

target_stage="${computed_next_stage}"
if [[ "${TARGET_STAGE}" != "NEXT" ]]; then
  target_stage="${TARGET_STAGE}"
  if [[ "${target_stage}" != "${computed_next_stage}" ]]; then
    fail "target_stage_must_match_next expected=${computed_next_stage} actual=${target_stage}"
  fi
fi

case "${target_stage}" in
  CANARY) target_rollout_pct=5 ; target_status="ACTIVE" ; stage_suffix="canary" ;;
  RAMP_25) target_rollout_pct=25 ; target_status="ACTIVE" ; stage_suffix="ramp25" ;;
  RAMP_50) target_rollout_pct=50 ; target_status="ACTIVE" ; stage_suffix="ramp50" ;;
  PRODUCTION) target_rollout_pct=100 ; target_status="COMPLETED" ; stage_suffix="production" ;;
  *) fail "unsupported_target_stage value=${target_stage}" ;;
esac

if [[ "${target_stage}" == "PRODUCTION" ]]; then
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
    fail "production_requires_approved_status proposal_id=${PROPOSAL_ID} approval_status=${approval_status:-NONE}"
  fi
fi

target_release_state_id="builder_release_${PROPOSAL_ID}_${stage_suffix}"
target_release_state_id="${target_release_state_id:0:96}"
reason_code_release_stage_active=$((16#B13D0008))

if [[ "${EXECUTE}" == "0" ]]; then
  echo "CHECK_OK builder_controlled_launch_execute=preview proposal_id=${PROPOSAL_ID} from_stage=${current_stage} to_stage=${target_stage} current_release_state_id=${current_release_state_id} target_release_state_id=${target_release_state_id}"
  exit 0
fi

if [[ "${LAUNCH_EXECUTE_ACK}" != "YES" ]]; then
  fail "missing_explicit_ack expected=YES"
fi
if [[ -z "${LAUNCH_EXECUTE_IDEMPOTENCY_KEY}" ]]; then
  fail "missing_idempotency_key"
fi
if ! [[ "${LAUNCH_EXECUTE_IDEMPOTENCY_KEY}" =~ ^[A-Za-z0-9._:-]{8,128}$ ]]; then
  fail "invalid_idempotency_key_format expected_ascii_token_len_8_128"
fi
idem_sql="$(sql_quote "${LAUNCH_EXECUTE_IDEMPOTENCY_KEY}")"

existing_idem="$(
  run_psql -AtF '|' -c "
    SELECT release_state_id, stage, status
    FROM builder_release_states
    WHERE proposal_id = '${proposal_sql}' AND idempotency_key = '${idem_sql}'
    ORDER BY recorded_at DESC, release_row_id DESC
    LIMIT 1;
  "
)"
if [[ -n "${existing_idem}" ]]; then
  IFS='|' read -r ex_release_state_id ex_stage ex_status <<< "${existing_idem}"
  echo "CHECK_OK builder_controlled_launch_execute=idempotent_reuse proposal_id=${PROPOSAL_ID} to_stage=${ex_stage} release_state_id=${ex_release_state_id} status=${ex_status} idempotency_key=${LAUNCH_EXECUTE_IDEMPOTENCY_KEY}"
  exit 0
fi

# Re-check latest release-state right before write (fail-closed on races).
latest_release_state_id="$(
  run_psql -Atqc "
    SELECT release_state_id
    FROM builder_release_states
    WHERE proposal_id = '${proposal_sql}'
    ORDER BY recorded_at DESC, release_row_id DESC
    LIMIT 1;
  "
)"
if [[ "${latest_release_state_id}" != "${current_release_state_id}" ]]; then
  fail "release_state_race_detected expected_latest=${current_release_state_id} actual_latest=${latest_release_state_id}"
fi

next_release_row_id="$(
  run_psql -Atqc "SELECT COALESCE(MAX(release_row_id), 0) + 1 FROM builder_release_states;"
)"
if [[ -z "${next_release_row_id}" ]]; then
  fail "unable_to_compute_release_row_id"
fi

now_ns="$(( $(date +%s) * 1000000000 ))"
rollback_hook_sql="$(sql_quote "${current_rollback_hook}")"
rollback_ready_sql="FALSE"
if [[ "${current_rollback_ready}" == "t" || "${current_rollback_ready}" == "true" || "${current_rollback_ready}" == "TRUE" ]]; then
  rollback_ready_sql="TRUE"
fi
target_release_state_id_sql="$(sql_quote "${target_release_state_id}")"

run_psql -c "
INSERT INTO builder_release_states (
  release_row_id,
  schema_version,
  release_state_id,
  proposal_id,
  stage,
  stage_rollout_pct,
  status,
  rollback_hook,
  rollback_ready,
  reason_code,
  recorded_at,
  idempotency_key
) VALUES (
  ${next_release_row_id},
  1,
  '${target_release_state_id_sql}',
  '${proposal_sql}',
  '${target_stage}',
  ${target_rollout_pct},
  '${target_status}',
  '${rollback_hook_sql}',
  ${rollback_ready_sql},
  ${reason_code_release_stage_active},
  ${now_ns},
  '${idem_sql}'
);
"

echo "CHECK_OK builder_controlled_launch_execute=executed proposal_id=${PROPOSAL_ID} from_stage=${current_stage} to_stage=${target_stage} release_state_id=${target_release_state_id} idempotency_key=${LAUNCH_EXECUTE_IDEMPOTENCY_KEY}"
