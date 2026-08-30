#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

EXPERIMENTS_EXECUTED=false
DEFER_HOSTED_RECEIPTS=false
CANDIDATES=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --experiments-executed)
      EXPERIMENTS_EXECUTED=true
      shift
      ;;
    --defer-hosted-receipts)
      DEFER_HOSTED_RECEIPTS=true
      shift
      ;;
    --candidates)
      [[ $# -ge 2 ]] || break
      CANDIDATES="$2"
      shift 2
      ;;
    *)
      break
      ;;
  esac
done
if [[ $# -ne 0 || "$EXPERIMENTS_EXECUTED" != true ]]; then
  printf '%s\n' \
    'usage: ./release/check.sh --experiments-executed [--candidates <dir>] [--defer-hosted-receipts]' >&2
  exit 2
fi

python3 -m unittest -v \
  release.test_qualify \
  release.test_isolate_experiments \
  release.test_orchestrate \
  release.test_publication
ORCHESTRATION_ARGS=()
if [[ "$DEFER_HOSTED_RECEIPTS" == true ]]; then
  ORCHESTRATION_ARGS+=(--defer-hosted-receipts)
fi
python3 release/orchestrate.py qualify "${ORCHESTRATION_ARGS[@]}"
python3 release/publication.py qualify --out .azimuth/release
if [[ -n "$CANDIDATES" ]]; then
  python3 release/qualify.py --candidates "$CANDIDATES"
else
  python3 release/qualify.py --allow-dirty
fi
python3 release/isolate_experiments.py --experiments-executed
cargo run --quiet --manifest-path tools/azimuth/Cargo.toml -- validate \
  --model azimuth/model \
  --standards azimuth/standards/verification.md \
  --manifest .azimuth/release/linkage.json \
  --manifest .azimuth/release/experimental-isolation-linkage.json \
  --manifest .azimuth/release/orchestration-linkage.json \
  --manifest .azimuth/release/publication-linkage.json \
  --manifest .azimuth/release/private-deployment-linkage.json
