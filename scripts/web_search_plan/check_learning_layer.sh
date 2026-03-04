#!/usr/bin/env bash
set -euo pipefail

cargo test -p selene_os --lib web_search_plan::learn::learn_tests --quiet
cargo test -p selene_os --lib web_search_plan::learn::learn_parity_tests --quiet
