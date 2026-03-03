#!/usr/bin/env bash
set -euo pipefail

cargo test -p selene_os web_search_plan::learning::learning_tests --quiet
cargo test -p selene_os web_search_plan::learn::learn_tests --quiet
