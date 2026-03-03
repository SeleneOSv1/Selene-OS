#!/usr/bin/env bash
set -euo pipefail

cargo test -p selene_os web_search_plan::web_provider::web_provider_tests --quiet
