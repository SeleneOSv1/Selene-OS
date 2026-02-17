#!/usr/bin/env bash
set -euo pipefail

LABEL="${LABEL:-com.selene.builder.production_soak_runner}"
REQUIRE_LOADED="${REQUIRE_LOADED:-0}"

fail() {
  echo "PRODUCTION_SOAK_LAUNCHD_STATUS_FAIL:$1" >&2
  exit 1
}

if [[ "${REQUIRE_LOADED}" != "0" && "${REQUIRE_LOADED}" != "1" ]]; then
  fail "invalid_REQUIRE_LOADED expected=0_or_1 actual=${REQUIRE_LOADED}"
fi
if ! command -v launchctl >/dev/null 2>&1; then
  fail "launchctl_not_found"
fi

domains=("gui/$(id -u)" "user/$(id -u)")
active_domain=""
print_output=""
for domain in "${domains[@]}"; do
  set +e
  candidate_output="$(launchctl print "${domain}/${LABEL}" 2>&1)"
  rc=$?
  set -e
  if [[ ${rc} -eq 0 ]]; then
    active_domain="${domain}"
    print_output="${candidate_output}"
    break
  fi
done

if [[ -z "${active_domain}" ]]; then
  if [[ "${REQUIRE_LOADED}" == "1" ]]; then
    fail "not_loaded label=${LABEL}"
  fi
  echo "CHECK_OK builder_production_soak_launchd_status=not_loaded label=${LABEL}"
  exit 0
fi

state_line="$(printf "%s\n" "${print_output}" | rg -n "state =" -m 1 || true)"
exit_line="$(printf "%s\n" "${print_output}" | rg -n "last exit code" -m 1 || true)"
echo "CHECK_OK builder_production_soak_launchd_status=loaded label=${LABEL} domain=${active_domain}"
if [[ -n "${state_line}" ]]; then
  printf "%s\n" "${state_line}"
fi
if [[ -n "${exit_line}" ]]; then
  printf "%s\n" "${exit_line}"
fi
