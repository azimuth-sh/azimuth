#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../.."
REPO_ROOT="$PWD"
EXPERIMENT_TMP="$(mktemp -d "${TMPDIR:-/tmp}/azimuth-adapters.XXXXXX")"
trap 'rm -rf "$EXPERIMENT_TMP"' EXIT

python3 experiments/adapter-capabilities/generate.py init "$EXPERIMENT_TMP/fixture"
cargo build --quiet --manifest-path tools/azimuth/Cargo.toml
AZIMUTH="$REPO_ROOT/tools/azimuth/target/debug/azimuth"
FIXTURE="$EXPERIMENT_TMP/fixture"

expect_status() {
  local expected="$1"
  shift
  set +e
  "$@" >"$EXPERIMENT_TMP/stdout" 2>"$EXPERIMENT_TMP/stderr"
  local actual="$?"
  set -e
  if [[ "$actual" -ne "$expected" ]]; then
    printf 'expected exit %s, got %s: %s\n' "$expected" "$actual" "$*" >&2
    sed -n '1,120p' "$EXPERIMENT_TMP/stdout" >&2
    sed -n '1,120p' "$EXPERIMENT_TMP/stderr" >&2
    exit 1
  fi
}

plan() {
  local request="$1"
  local config="$2"
  shift 2
  "$AZIMUTH" run plan \
    --request "$request" \
    --model "$FIXTURE/model" \
    --standards "$FIXTURE/standards.md" \
    --workspace "$FIXTURE/workspace.json" \
    --manifest "$FIXTURE/manifest.json" \
    --config "$config" \
    "$@"
}

invoke_raw_importer() {
  local request="$1"
  env -i MODE=normal SYNTHETIC_LITERAL=exact-value \
    /usr/bin/python3 experiments/adapter-capabilities/adapters/importer.py <"$request"
}

# Both configured executables complete the real public description handshake.
expect_status 0 "$AZIMUTH" adapter verify --config "$FIXTURE/adapters.json"
test ! -s "$EXPERIMENT_TMP/stdout"

# Configuration and returned descriptor drift fail in their distinct exit classes.
expect_status 2 "$AZIMUTH" adapter verify \
  --config "$FIXTURE/adapters-configuration-drift.json"
test ! -s "$EXPERIMENT_TMP/stdout"
expect_status 1 "$AZIMUTH" adapter verify \
  --config "$FIXTURE/adapters-descriptor-drift.json"

# Plan stdout is byte-identical to atomic file output, and partial-model selection is absent.
expect_status 0 plan "$FIXTURE/request-execute.json" "$FIXTURE/adapters.json"
cp "$EXPERIMENT_TMP/stdout" "$FIXTURE/launch-execute.json"
expect_status 0 plan "$FIXTURE/request-execute.json" "$FIXTURE/adapters.json" \
  --out "$FIXTURE/launch-execute-out.json"
test ! -s "$EXPERIMENT_TMP/stdout"
cmp "$FIXTURE/launch-execute.json" "$FIXTURE/launch-execute-out.json"
expect_status 2 plan "$FIXTURE/request-execute.json" "$FIXTURE/adapters.json" \
  --only 'synthetic/**'

expect_status 0 plan "$FIXTURE/request-import.json" "$FIXTURE/adapters.json"
cp "$EXPERIMENT_TMP/stdout" "$FIXTURE/launch-import.json"
expect_status 0 plan "$FIXTURE/request-import.json" "$FIXTURE/adapters.json" \
  --out "$FIXTURE/launch-import-out.json"
cmp "$FIXTURE/launch-import.json" "$FIXTURE/launch-import-out.json"

# Execute stdout and --out are exact, and the normalized bundle passes both standalone commands.
expect_status 0 "$AZIMUTH" run execute \
  --plan "$FIXTURE/launch-execute.json" --config "$FIXTURE/adapters.json"
cp "$EXPERIMENT_TMP/stdout" "$FIXTURE/executed.json"
expect_status 0 "$AZIMUTH" run execute \
  --plan "$FIXTURE/launch-execute.json" --config "$FIXTURE/adapters.json" \
  --out "$FIXTURE/executed-out.json"
test ! -s "$EXPERIMENT_TMP/stdout"
cmp "$FIXTURE/executed.json" "$FIXTURE/executed-out.json"
expect_status 0 "$AZIMUTH" run verify --bundle "$FIXTURE/executed.json"
grep -q '^protocol-consistent$' "$EXPERIMENT_TMP/stdout"
expect_status 0 "$AZIMUTH" run inspect \
  --bundle "$FIXTURE/executed.json" --format json
cp "$EXPERIMENT_TMP/stdout" "$FIXTURE/inspection.json"
expect_status 0 "$AZIMUTH" run inspect \
  --bundle "$FIXTURE/executed.json" --format json \
  --out "$FIXTURE/inspection-out.json"
cmp "$FIXTURE/inspection.json" "$FIXTURE/inspection-out.json"

# The import adapter receives staged exact bytes. Relocating identical content changes no identity.
expect_status 0 "$AZIMUTH" run import \
  --plan "$FIXTURE/launch-import.json" \
  --input "native-report=$FIXTURE/native-report.json" \
  --config "$FIXTURE/adapters.json"
cp "$EXPERIMENT_TMP/stdout" "$FIXTURE/imported.json"
expect_status 0 "$AZIMUTH" run import \
  --plan "$FIXTURE/launch-import.json" \
  --input "native-report=$FIXTURE/relocated/same-report.json" \
  --config "$FIXTURE/adapters.json" --out "$FIXTURE/imported-relocated.json"
cmp "$FIXTURE/imported.json" "$FIXTURE/imported-relocated.json"

# A stateless import correction consumes the verified terminal predecessor and advances the chain.
expect_status 0 "$AZIMUTH" run import \
  --plan "$FIXTURE/launch-import.json" \
  --input "native-report=$FIXTURE/later-report.json" \
  --predecessor "$FIXTURE/imported.json" \
  --config "$FIXTURE/adapters.json" --out "$FIXTURE/correction.json"
expect_status 0 "$AZIMUTH" run verify \
  --bundle "$FIXTURE/correction.json" --bundle "$FIXTURE/imported.json"
expect_status 0 "$AZIMUTH" run inspect \
  --bundle "$FIXTURE/imported.json" --bundle "$FIXTURE/correction.json" --format json

# The adapter independently rejects missing, malformed or stale predecessor request identities.
python3 experiments/adapter-capabilities/generate.py invalid-requests \
  "$FIXTURE" "$REPO_ROOT/experiments/adapter-capabilities/adapters/runtime.py"
for name in predecessors-absent predecessors-malformed predecessor-identity-stale; do
  expect_status 1 invoke_raw_importer "$FIXTURE/request-$name.json"
  test ! -s "$EXPERIMENT_TMP/stdout"
done

# Capability substitution is either rejected as stale or becomes a new launch and Run identity.
python3 experiments/adapter-capabilities/generate.py launches \
  "$FIXTURE/launch-execute.json" "$FIXTURE/adapters.json" "$FIXTURE"
expect_status 2 "$AZIMUTH" run execute \
  --plan "$FIXTURE/substitution-stale.json" --config "$FIXTURE/adapters.json" \
  --out "$FIXTURE/must-not-exist-stale.json"
test ! -e "$FIXTURE/must-not-exist-stale.json"
expect_status 0 "$AZIMUTH" run execute \
  --plan "$FIXTURE/substitution-reidentified.json" --config "$FIXTURE/adapters.json" \
  --out "$FIXTURE/substituted.json"

# One dual-role activity remains an Observation and a successful Challenge finding record.
expect_status 0 "$AZIMUTH" run execute \
  --plan "$FIXTURE/dual-role.json" --config "$FIXTURE/adapters.json" \
  --out "$FIXTURE/dual-role-bundle.json"
expect_status 0 "$AZIMUTH" run verify --bundle "$FIXTURE/dual-role-bundle.json"
expect_status 0 "$AZIMUTH" run inspect \
  --bundle "$FIXTURE/dual-role-bundle.json" --format json

# Violations and incomplete terminal facts are successful protocol exchanges, not host failures.
for mode in violated partial cancelled timed-out-fact; do
  config="$FIXTURE/adapters-$mode.json"
  launch="$FIXTURE/launch-$mode.json"
  bundle="$FIXTURE/bundle-$mode.json"
  expect_status 0 plan "$FIXTURE/request-execute.json" "$config" --out "$launch"
  expect_status 0 "$AZIMUTH" run execute \
    --plan "$launch" --config "$config" --out "$bundle"
  expect_status 0 "$AZIMUTH" run verify --bundle "$bundle"
done

# Process and stream failures are exit one; response schema failures are exit two. None publish.
for mode in nonzero hang stdout-overflow stderr-overflow extra-output; do
  config="$FIXTURE/adapters-$mode.json"
  launch="$FIXTURE/launch-$mode.json"
  output="$FIXTURE/must-not-exist-$mode.json"
  expect_status 0 plan "$FIXTURE/request-execute.json" "$config" --out "$launch"
  expect_status 1 "$AZIMUTH" run execute \
    --plan "$launch" --config "$config" --out "$output"
  test ! -e "$output"
done
for mode in malformed schema; do
  config="$FIXTURE/adapters-$mode.json"
  launch="$FIXTURE/launch-$mode.json"
  output="$FIXTURE/must-not-exist-$mode.json"
  expect_status 0 plan "$FIXTURE/request-execute.json" "$config" --out "$launch"
  expect_status 2 "$AZIMUTH" run execute \
    --plan "$launch" --config "$config" --out "$output"
  test ! -e "$output"
done

python3 - "$FIXTURE" <<'PY'
import hashlib
import json
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
load = lambda name: json.load(open(root / name, encoding="utf-8"))

executed = load("executed.json")
substituted = load("substituted.json")
assert executed["run_id"] != substituted["run_id"]
assert executed["provenance"]["adapter"]["routes"] != (
    substituted["provenance"]["adapter"]["routes"]
)

imported = load("imported.json")
identity = imported["provenance"]["adapter"]["import_inputs"][0]
content = (root / "native-report.json").read_bytes()
assert identity == {
    "id": "native-report",
    "digest": "sha256:" + hashlib.sha256(content).hexdigest(),
    "size_bytes": len(content),
}
correction = load("correction.json")
assert correction["bundle_revision"] == 1
assert correction["corrects"] == imported["bundle_fingerprint"]
assert correction["provenance"]["adapter"]["import_inputs"] != [identity]

dual = load("dual-role-bundle.json")
assert len(dual["check_executions"]) == 1
assert len(dual["challenger_executions"]) == 1
selection = dual["plan"]["challenges"][0]
selection_preimage = {
    "format": "azimuth-challenge-selection-identity",
    "version": 1,
    "challenger_fingerprint": selection["challenger"]["fingerprint"],
    "target_kind": selection["target"]["kind"],
    "target_fingerprint": selection["target"]["fingerprint"],
}
expected_selection = "challenge/" + hashlib.sha256(
    json.dumps(selection_preimage, separators=(",", ":"), sort_keys=True).encode()
).hexdigest()
assert selection["id"] == expected_selection
assert selection["lane"] == "gate"
scope = selection["scope"]
scope_preimage = {
    "format": "azimuth-challenge-scope-fingerprint",
    "version": 1,
    "anchors": scope["anchors"],
    "inputs": scope["inputs"],
}
assert scope["fingerprint"] == "sha256:" + hashlib.sha256(
    json.dumps(scope_preimage, separators=(",", ":"), sort_keys=True).encode()
).hexdigest()
assert [(item["kind"], item["id"]) for item in scope["anchors"]] == [
    ("claim", "synthetic/behavior#works")
]
assert [(item["kind"], item["id"]) for item in scope["inputs"]] == [
    ("claim", "synthetic/behavior#works"),
    ("claim-judgment", "synthetic/behavior#works"),
    ("realization", "synthetic|rust-symbol|behavior::execute"),
    ("policy", "synthetic/claim-judgment-policy"),
]
challenge_route = dual["provenance"]["adapter"]["routes"][1]
assert challenge_route["selection"] == {"kind": "challenge", "id": selection["id"]}
assert [
    (item["kind"], item["id"], item["fingerprint"])
    for item in challenge_route["inputs"]
] == [
    (item["kind"], item["id"], item["fingerprint"])
    for item in scope["inputs"]
    if item["kind"] == "realization"
]
assert challenge_route["inputs"][0]["source"] == {
    "kind": "source",
    "file": "src/behavior.rs",
    "language": "rust-symbol",
    "site": "behavior::execute",
}
check_activity = dual["check_executions"][0]["units"][0]["attempts"][0]["activity"]
challenge_activity = dual["challenger_executions"][0]["units"][0]["attempts"][0]["activity"]
assert check_activity == challenge_activity == "shared-work"
assert dual["check_executions"][0]["observations"][0]["outcome"] == "satisfied"
challenge = dual["challenger_executions"][0]
assert challenge["result"]["outcome"] == "findings"
assert challenge["result"]["objections"] == ["challenge/synthetic-finding"]
finding = dual["diagnostics"][0]
assert finding["id"] == "challenge/synthetic-finding"
assert finding["class"] == "objection"
assert finding["scope"]["challenger_fingerprint"] == challenge["challenger"]["fingerprint"]
assert finding["scope"]["target_fingerprint"] == challenge["target"]["fingerprint"]

expected = {
    "violated": ("complete", "violated"),
    "partial": ("partial", None),
    "cancelled": ("cancelled", None),
    "timed-out-fact": ("timed-out", "inconclusive"),
}
for name, (status, outcome) in expected.items():
    bundle = load(f"bundle-{name}.json")
    assert bundle["status"] == status
    actual = bundle["check_executions"]
    assert (actual[0]["observations"][0]["outcome"] if actual else None) == outcome
PY

echo "adapter capability conformance passed"
