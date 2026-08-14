#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

if [[ $# -ne 1 || "$1" != "--experiments-executed" ]]; then
  printf '%s\n' 'usage: ./release/check.sh --experiments-executed' >&2
  exit 2
fi

python3 -m unittest -v release.test_qualify release.test_isolate_experiments
python3 release/qualify.py --allow-dirty
python3 release/isolate_experiments.py --experiments-executed
cargo run --quiet --manifest-path tools/azimuth/Cargo.toml -- check \
  --model azimuth/model \
  --standards azimuth/standards/verification.md \
  --manifest .azimuth/release/linkage.json \
  --manifest .azimuth/release/experimental-isolation-linkage.json \
  --manifest .azimuth/release/private-deployment-linkage.json
