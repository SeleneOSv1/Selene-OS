#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ENV_FILE="${ENV_FILE:-${ROOT_DIR}/.dev/db.env}"
ADMIN_ENV_FILE="${ADMIN_ENV_FILE:-${ROOT_DIR}/.dev/postgres/admin.env}"

if [[ ! -f "${ENV_FILE}" ]]; then
  echo "Missing env file: ${ENV_FILE}" >&2
  exit 1
fi

if [[ ! -f "${ADMIN_ENV_FILE}" ]]; then
  echo "Missing admin env file: ${ADMIN_ENV_FILE}" >&2
  exit 1
fi

# shellcheck disable=SC1090
source "${ENV_FILE}"
# shellcheck disable=SC1090
source "${ADMIN_ENV_FILE}"

PSQL_BIN="/opt/homebrew/opt/libpq/bin/psql"
if [[ ! -x "${PSQL_BIN}" ]]; then
  PSQL_BIN="$(command -v psql || true)"
fi
if [[ -z "${PSQL_BIN}" || ! -x "${PSQL_BIN}" ]]; then
  echo "psql was not found." >&2
  exit 1
fi

admin_psql() {
  PGPASSWORD="${PG_SUPERPASS}" \
    "${PSQL_BIN}" \
    -h "${PGHOST}" \
    -p "${PGPORT}" \
    -U "${PG_SUPERUSER}" \
    -d "${PGDATABASE}" \
    -v ON_ERROR_STOP=1 \
    "$@"
}

extra_dbs="$(admin_psql -Atqc "SELECT datname FROM pg_database WHERE datistemplate=false AND datname <> '${PGDATABASE}' ORDER BY datname;")"
extra_roles="$(admin_psql -Atqc "SELECT rolname FROM pg_roles WHERE rolname !~ '^pg_' AND rolname NOT IN ('${PG_SUPERUSER}','${PGUSER}') ORDER BY rolname;")"
active_non_selene="$(admin_psql -Atqc "SELECT datname || '|' || usename || '|' || application_name FROM pg_stat_activity WHERE pid <> pg_backend_pid() AND (datname IS DISTINCT FROM '${PGDATABASE}' OR usename NOT IN ('${PG_SUPERUSER}','${PGUSER}')) ORDER BY datname, usename;")"

echo "PGHOST=${PGHOST}"
echo "PGPORT=${PGPORT}"
echo "APP_DB=${PGDATABASE}"
echo "APP_USER=${PGUSER}"
echo "ADMIN_USER=${PG_SUPERUSER}"

if [[ -n "${extra_dbs}" ]]; then
  echo "Isolation check failed: extra non-template DBs exist:"
  echo "${extra_dbs}"
  exit 1
fi

if [[ -n "${extra_roles}" ]]; then
  echo "Isolation check failed: extra non-system roles exist:"
  echo "${extra_roles}"
  exit 1
fi

if [[ -n "${active_non_selene}" ]]; then
  echo "Isolation check failed: non-Selene active connections exist:"
  echo "${active_non_selene}"
  exit 1
fi

echo "Isolation check passed."
