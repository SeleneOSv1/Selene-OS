#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ENV_FILE="${ENV_FILE:-${ROOT_DIR}/.dev/db.env}"
MIGRATIONS_DIR="${MIGRATIONS_DIR:-${ROOT_DIR}/crates/selene_storage/migrations}"

if [[ ! -f "${ENV_FILE}" ]]; then
  echo "Missing env file: ${ENV_FILE}" >&2
  echo "Run: ${ROOT_DIR}/scripts/dev_postgres_setup.sh" >&2
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

if [[ ! -d "${MIGRATIONS_DIR}" ]]; then
  echo "Migrations directory not found: ${MIGRATIONS_DIR}" >&2
  exit 1
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

run_psql -c "CREATE TABLE IF NOT EXISTS public.schema_migrations (
  file_name TEXT PRIMARY KEY,
  applied_at TIMESTAMPTZ NOT NULL DEFAULT now()
);"

# If the schema already exists but migration tracking doesn't, backfill entries once.
existing_count="$(run_psql -Atqc "SELECT count(*)::text FROM public.schema_migrations;")"
if [[ "${existing_count}" == "0" ]]; then
  has_foundation="$(run_psql -Atqc "SELECT CASE WHEN to_regclass('public.identities') IS NULL THEN '0' ELSE '1' END;")"
  if [[ "${has_foundation}" == "1" ]]; then
    for file in $(ls "${MIGRATIONS_DIR}"/*.sql | sort); do
      name="$(basename "${file}")"
      run_psql -c "INSERT INTO public.schema_migrations(file_name) VALUES ('${name}') ON CONFLICT (file_name) DO NOTHING;" >/dev/null
    done
    echo "Backfilled schema_migrations from existing schema state."
  fi
fi

for file in $(ls "${MIGRATIONS_DIR}"/*.sql | sort); do
  name="$(basename "${file}")"
  applied="$(run_psql -Atqc "SELECT CASE WHEN EXISTS (SELECT 1 FROM public.schema_migrations WHERE file_name='${name}') THEN '1' ELSE '0' END;")"
  if [[ "${applied}" == "1" ]]; then
    echo "Skipping ${name} (already applied)"
    continue
  fi

  echo "Applying ${name}"
  run_psql -f "${file}" >/dev/null
  run_psql -c "INSERT INTO public.schema_migrations(file_name) VALUES ('${name}') ON CONFLICT (file_name) DO NOTHING;" >/dev/null
done

echo "All migrations applied to ${PGDATABASE}."
