#!/usr/bin/env bash
set -euo pipefail

cargo test -p selene_os web_search_plan::gap_closers::gap_closers_tests --quiet
