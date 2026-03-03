#!/usr/bin/env bash
set -euo pipefail

cargo test -p selene_os web_search_plan::tests::test_unknown_reason_code_fails --quiet
