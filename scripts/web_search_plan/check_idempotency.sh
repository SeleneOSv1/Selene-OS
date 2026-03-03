#!/usr/bin/env bash
set -euo pipefail

cargo test -p selene_os web_search_plan::tests::test_idempotency_registry_foundation_entries_present --quiet
