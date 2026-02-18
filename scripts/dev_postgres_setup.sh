#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

PG_FORMULA="${PG_FORMULA:-postgresql@16}"
PGHOST="${PGHOST:-127.0.0.1}"
PGPORT="${PGPORT:-55432}"
PG_SUPERUSER="${PG_SUPERUSER:-selene_admin}"
PGUSER="${PGUSER:-selene_dev}"
PGDATABASE="${PGDATABASE:-selene_os_dev}"
AUTO_MIGRATE="${AUTO_MIGRATE:-1}"

CLUSTER_BASE="${CLUSTER_BASE:-${ROOT_DIR}/.dev/postgres}"
CLUSTER_DIR="${CLUSTER_DIR:-${CLUSTER_BASE}/cluster}"
SOCKET_DIR="${SOCKET_DIR:-${CLUSTER_BASE}/run}"
LOG_FILE="${LOG_FILE:-${CLUSTER_BASE}/postgres.log}"
ADMIN_ENV_FILE="${ADMIN_ENV_FILE:-${CLUSTER_BASE}/admin.env}"
ENV_FILE="${ENV_FILE:-${ROOT_DIR}/.dev/db.env}"
PROJECT_ENV_FILE="${PROJECT_ENV_FILE:-${ROOT_DIR}/.env.local}"

if ! command -v brew >/dev/null 2>&1; then
  echo "Homebrew is required but was not found in PATH." >&2
  exit 1
fi

if ! brew ls --versions "${PG_FORMULA}" >/dev/null 2>&1; then
  echo "Installing ${PG_FORMULA}..."
  brew install "${PG_FORMULA}"
fi

FORMULA_PREFIX="$(brew --prefix "${PG_FORMULA}")"
LIBPQ_PREFIX="$(brew --prefix libpq 2>/dev/null || true)"
INITDB_BIN="${FORMULA_PREFIX}/bin/initdb"
PG_CTL_BIN="${FORMULA_PREFIX}/bin/pg_ctl"
PG_ISREADY_BIN="${FORMULA_PREFIX}/bin/pg_isready"
CREATEDB_BIN="${FORMULA_PREFIX}/bin/createdb"
DROPDB_BIN="${FORMULA_PREFIX}/bin/dropdb"
PSQL_BIN="${FORMULA_PREFIX}/bin/psql"

if [[ ! -x "${PSQL_BIN}" && -n "${LIBPQ_PREFIX}" ]]; then
  PSQL_BIN="${LIBPQ_PREFIX}/bin/psql"
fi

for bin in "${INITDB_BIN}" "${PG_CTL_BIN}" "${PG_ISREADY_BIN}" "${CREATEDB_BIN}" "${DROPDB_BIN}" "${PSQL_BIN}"; do
  if [[ ! -x "${bin}" ]]; then
    echo "Missing PostgreSQL binary: ${bin}" >&2
    exit 1
  fi
done

mkdir -p "${CLUSTER_BASE}" "$(dirname "${ENV_FILE}")"
chmod 700 "${CLUSTER_BASE}"

if [[ -f "${ADMIN_ENV_FILE}" ]]; then
  # shellcheck disable=SC1090
  source "${ADMIN_ENV_FILE}"
fi

if [[ -z "${PG_SUPERPASS:-}" ]]; then
  PG_SUPERPASS="$(openssl rand -hex 24)"
fi

if [[ ! -f "${CLUSTER_DIR}/PG_VERSION" ]]; then
  rm -rf "${CLUSTER_DIR}" "${SOCKET_DIR}"
  mkdir -p "${CLUSTER_DIR}" "${SOCKET_DIR}"
  chmod 700 "${CLUSTER_DIR}" "${SOCKET_DIR}"

  pwfile="$(mktemp)"
  trap 'rm -f "${pwfile}"' EXIT
  printf '%s\n' "${PG_SUPERPASS}" > "${pwfile}"

  "${INITDB_BIN}" \
    -D "${CLUSTER_DIR}" \
    -U "${PG_SUPERUSER}" \
    --pwfile="${pwfile}" \
    --auth-local=scram-sha-256 \
    --auth-host=scram-sha-256 \
    -E UTF8 >/dev/null

  cat > "${CLUSTER_DIR}/selene_isolated.conf" <<EOF
listen_addresses = '${PGHOST}'
port = ${PGPORT}
unix_socket_directories = '${SOCKET_DIR}'
password_encryption = 'scram-sha-256'
logging_collector = on
log_destination = 'stderr'
log_directory = '${CLUSTER_BASE}'
log_filename = 'postgres.log'
EOF

  if ! grep -q "include_if_exists = 'selene_isolated.conf'" "${CLUSTER_DIR}/postgresql.conf"; then
    echo "include_if_exists = 'selene_isolated.conf'" >> "${CLUSTER_DIR}/postgresql.conf"
  fi

  cat > "${CLUSTER_DIR}/pg_hba.conf" <<EOF
local   all             all                                     scram-sha-256
host    all             all             127.0.0.1/32            scram-sha-256
host    all             all             ::1/128                 scram-sha-256
EOF
fi

mkdir -p "${SOCKET_DIR}"
chmod 700 "${SOCKET_DIR}"

if ! "${PG_CTL_BIN}" -D "${CLUSTER_DIR}" status >/dev/null 2>&1; then
  "${PG_CTL_BIN}" -D "${CLUSTER_DIR}" -l "${LOG_FILE}" start >/dev/null
fi

for _ in {1..30}; do
  if "${PG_ISREADY_BIN}" -h "${PGHOST}" -p "${PGPORT}" >/dev/null 2>&1; then
    break
  fi
  sleep 0.5
done

if ! "${PG_ISREADY_BIN}" -h "${PGHOST}" -p "${PGPORT}" >/dev/null 2>&1; then
  echo "Isolated PostgreSQL is not accepting connections at ${PGHOST}:${PGPORT}." >&2
  exit 1
fi

run_admin_psql() {
  PGPASSWORD="${PG_SUPERPASS}" \
    "${PSQL_BIN}" \
    -h "${PGHOST}" \
    -p "${PGPORT}" \
    -U "${PG_SUPERUSER}" \
    -v ON_ERROR_STOP=1 \
    "$@"
}

if [[ -z "${PGPASSWORD:-}" ]]; then
  PGPASSWORD="$(openssl rand -hex 24)"
fi

run_admin_psql -d postgres -c "DO \$do\$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname='${PGUSER}') THEN CREATE ROLE ${PGUSER} LOGIN PASSWORD '${PGPASSWORD}'; ELSE ALTER ROLE ${PGUSER} LOGIN PASSWORD '${PGPASSWORD}'; END IF; END \$do\$;"

if ! run_admin_psql -d postgres -Atqc "SELECT 1 FROM pg_database WHERE datname='${PGDATABASE}'" | grep -q 1; then
  PGPASSWORD="${PG_SUPERPASS}" "${CREATEDB_BIN}" -h "${PGHOST}" -p "${PGPORT}" -U "${PG_SUPERUSER}" -O "${PGUSER}" "${PGDATABASE}"
fi

run_admin_psql -d postgres -c "ALTER DATABASE ${PGDATABASE} OWNER TO ${PGUSER};"
run_admin_psql -d postgres -c "GRANT ALL PRIVILEGES ON DATABASE ${PGDATABASE} TO ${PGUSER};"

# Hard isolation cleanup: keep only the Selene app DB + templates.
for db in $(run_admin_psql -d "${PGDATABASE}" -Atqc "SELECT datname FROM pg_database WHERE datistemplate = false AND datname <> '${PGDATABASE}';"); do
  run_admin_psql -d "${PGDATABASE}" -Atqc "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname='${db}' AND pid <> pg_backend_pid();" >/dev/null || true
  PGPASSWORD="${PG_SUPERPASS}" "${DROPDB_BIN}" -h "${PGHOST}" -p "${PGPORT}" -U "${PG_SUPERUSER}" "${db}" || true
done

# Keep only Selene roles + built-in pg_* roles.
for role in $(run_admin_psql -d "${PGDATABASE}" -Atqc "SELECT rolname FROM pg_roles WHERE rolname !~ '^pg_' AND rolname NOT IN ('${PG_SUPERUSER}','${PGUSER}');"); do
  run_admin_psql -d "${PGDATABASE}" -c "DROP ROLE IF EXISTS ${role};" || true
done

cat > "${ADMIN_ENV_FILE}" <<EOF
PGHOST=${PGHOST}
PGPORT=${PGPORT}
PG_SUPERUSER=${PG_SUPERUSER}
PG_SUPERPASS=${PG_SUPERPASS}
EOF
chmod 600 "${ADMIN_ENV_FILE}"

cat > "${ENV_FILE}" <<EOF
PGHOST=${PGHOST}
PGPORT=${PGPORT}
PGUSER=${PGUSER}
PGPASSWORD=${PGPASSWORD}
PGDATABASE=${PGDATABASE}
DATABASE_URL=postgresql://${PGUSER}:${PGPASSWORD}@${PGHOST}:${PGPORT}/${PGDATABASE}
EOF
chmod 600 "${ENV_FILE}"
cp "${ENV_FILE}" "${PROJECT_ENV_FILE}"
chmod 600 "${PROJECT_ENV_FILE}"

echo "Isolated PostgreSQL is ready for Selene OS."
echo "Cluster dir: ${CLUSTER_DIR}"
echo "Socket dir: ${SOCKET_DIR}"
echo "App env: ${ENV_FILE}"
echo "Project env: ${PROJECT_ENV_FILE}"

if [[ "${AUTO_MIGRATE}" == "1" ]]; then
  ENV_FILE="${ENV_FILE}" "${ROOT_DIR}/scripts/dev_postgres_migrate.sh"
else
  echo "Next step: ${ROOT_DIR}/scripts/dev_postgres_migrate.sh"
fi
