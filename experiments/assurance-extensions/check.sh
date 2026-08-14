#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
ROOT="$PWD"
OUT="$ROOT/.azimuth/assurance-extensions"
IMPORT="$ROOT/tools/extractors/typescript/dist"
mkdir -p "$OUT"

(cd tools/extractors/typescript && npx tsc -p tsconfig.json)
node "$IMPORT/observation-cli.js" experiments/assurance-extensions/load-result.json \
  "$OUT/load.json" --root "$ROOT" --input experiments/assurance-extensions/fixture/load.js
node "$IMPORT/observation-cli.js" experiments/assurance-extensions/chaos-result.json \
  "$OUT/chaos.json" --root "$ROOT" --input experiments/assurance-extensions/fixture/broker-loss.yml
node "$IMPORT/sarif-cli.js" experiments/assurance-extensions/static-analysis.sarif \
  experiments/assurance-extensions/linkage.json "$OUT/sarif.json" --root "$ROOT"

cargo run --quiet --manifest-path tools/azimuth/Cargo.toml -- check \
  --model experiments/assurance-extensions/model \
  --standards experiments/assurance-extensions/standards/verification.md \
  --manifest experiments/assurance-extensions/linkage.json \
  --manifest "$OUT/load.json" --manifest "$OUT/chaos.json" --manifest "$OUT/sarif.json"
cargo run --quiet --manifest-path tools/azimuth/Cargo.toml -- export \
  --model experiments/assurance-extensions/model \
  --standards experiments/assurance-extensions/standards/verification.md \
  --manifest experiments/assurance-extensions/linkage.json \
  --manifest "$OUT/load.json" --manifest "$OUT/chaos.json" --manifest "$OUT/sarif.json" \
  --out "$OUT/model.json"

test "$(rg -c '\"id\": \"expected-load-20260811\"' "$OUT/model.json")" -eq 1
test "$(rg -c '\"id\": \"broker-loss-20260811\"' "$OUT/model.json")" -eq 1
test "$(rg -c '\"role\": \"challenge\"' "$OUT/model.json")" -eq 6
echo "assurance extension conformance passed"
