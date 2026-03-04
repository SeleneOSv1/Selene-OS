#!/usr/bin/env bash
set -euo pipefail

cargo test -p selene_os --lib web_search_plan::news::news_tests --quiet
