#!/usr/bin/env bash
set -euo pipefail

cargo test -p selene_engines ph1vision::tests::at_vision_media_ --quiet
cargo test -p selene_engines ph1vision --quiet
cargo test -p selene_os ph1vision --quiet
