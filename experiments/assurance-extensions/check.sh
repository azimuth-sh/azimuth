#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
python3 -m json.tool \
  experiments/assurance-extensions/static-analysis.sarif >/dev/null
python3 -c \
  'import json, sys; report=json.load(open(sys.argv[1])); assert report["version"] == "2.1.0"; assert report["runs"][0]["results"] == []' \
  experiments/assurance-extensions/static-analysis.sarif
echo "neutral analyzer fixture passed"
