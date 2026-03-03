#!/usr/bin/env bash
set -euo pipefail

cargo test -p selene_os web_search_plan::cache::cache_tests --quiet
cargo test -p selene_os web_search_plan::parallel::parallel_tests --quiet
