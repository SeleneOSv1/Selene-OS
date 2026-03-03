#!/usr/bin/env bash
set -euo pipefail

cargo test -p selene_os web_search_plan::structured::structured_tests --quiet
