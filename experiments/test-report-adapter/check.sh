#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../.."
cargo build --quiet --manifest-path tools/azimuth/Cargo.toml
python3 -m unittest discover -s experiments/test-report-adapter -p 'test_*.py' -v
