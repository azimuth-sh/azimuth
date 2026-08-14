#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

python3 -m unittest -v release/test_qualify.py
python3 release/qualify.py --allow-dirty
cargo run --quiet --manifest-path tools/azimuth/Cargo.toml -- check \
  --model azimuth/model \
  --standards azimuth/standards/verification.md \
  --manifest .azimuth/release/linkage.json
