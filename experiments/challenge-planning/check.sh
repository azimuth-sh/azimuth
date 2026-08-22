#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../.."
REPO_ROOT="$PWD"
EXPERIMENT_TMP="$(mktemp -d "${TMPDIR:-/tmp}/azimuth-challenge-planning.XXXXXX")"
trap 'rm -rf "$EXPERIMENT_TMP"' EXIT

python3 experiments/challenge-planning/generate.py init "$EXPERIMENT_TMP/fixture"
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
    sed -n '1,160p' "$EXPERIMENT_TMP/stdout" >&2
    sed -n '1,160p' "$EXPERIMENT_TMP/stderr" >&2
    exit 1
  fi
}

model_args=(
  --model "$FIXTURE/model"
  --standards "$FIXTURE/standards.md"
  --workspace "$FIXTURE/workspace.json"
  --manifest "$FIXTURE/manifest.json"
)

plan() {
  local request="$1"
  local config="$2"
  shift 2
  "$AZIMUTH" run plan --request "$request" "${model_args[@]}" --config "$config" "$@"
}

# Public export supplies the expected decision fingerprints; the generator only seals those bytes.
expect_status 0 "$AZIMUTH" export "${model_args[@]}" --out "$FIXTURE/initial-export.json"
python3 experiments/challenge-planning/generate.py seal "$FIXTURE"
expect_status 0 "$AZIMUTH" validate "${model_args[@]}"
expect_status 0 "$AZIMUTH" export "${model_args[@]}" --out "$FIXTURE/export.json"
expect_status 0 "$AZIMUTH" adapter verify --config "$FIXTURE/adapters-survivor.json"

# The all-selector launch is deterministic and contains exact merged semantic and source accounts.
expect_status 0 plan "$FIXTURE/request-all.json" "$FIXTURE/adapters-killed.json"
cp "$EXPERIMENT_TMP/stdout" "$FIXTURE/launch-all.json"
expect_status 0 plan "$FIXTURE/request-all.json" "$FIXTURE/adapters-killed.json" \
  --out "$FIXTURE/launch-all-out.json"
test ! -s "$EXPERIMENT_TMP/stdout"
cmp "$FIXTURE/launch-all.json" "$FIXTURE/launch-all-out.json"

# Required form, candidate cap, exact context and capability-form failures are semantic exit one.
for item in \
  "required-form:request-required-form.json:adapters-killed.json" \
  "cap:request-cap.json:adapters-killed.json" \
  "context:request-context.json:adapters-killed.json" \
  "form:request-fault.json:adapters-form.json"; do
  IFS=: read -r name request config <<<"$item"
  output="$FIXTURE/must-not-exist-$name.json"
  expect_status 1 plan "$FIXTURE/$request" "$FIXTURE/$config" --out "$output"
  test ! -e "$output"
done

# A real authored selector that resolves to zero candidates is a planning error, not a runtime
# zero-mutant fact. The deliberately unresolved model is isolated from the conformance model.
zero_output="$FIXTURE/must-not-exist-zero-resolution.json"
expect_status 1 "$AZIMUTH" run plan --request "$FIXTURE/request-zero-resolution.json" \
  --model "$FIXTURE/model-zero" --standards "$FIXTURE/standards.md" \
  --workspace "$FIXTURE/workspace.json" --manifest "$FIXTURE/manifest.json" \
  --config "$FIXTURE/adapters-killed.json" --out "$zero_output"
test ! -e "$zero_output"

# Mutation survivor, killed-set and zero-mutant runs remain distinct Challenge facts.
for mode in survivor killed zero; do
  config="$FIXTURE/adapters-$mode.json"
  launch="$FIXTURE/launch-$mode.json"
  bundle="$FIXTURE/bundle-$mode.json"
  expect_status 0 plan "$FIXTURE/request-mutation.json" "$config" --out "$launch"
  expect_status 0 "$AZIMUTH" run execute --plan "$launch" --config "$config" --out "$bundle"
  expect_status 0 "$AZIMUTH" run verify --bundle "$bundle"
  expect_status 0 "$AZIMUTH" run inspect --bundle "$bundle" --format json
done

# Each Challenger derives only its own native outcome vocabulary from its configured form.
for mode in broad-warning broad-clean broad-unsupported; do
  config="$FIXTURE/adapters-$mode.json"
  expect_status 0 plan "$FIXTURE/request-analyzers.json" "$config" \
    --out "$FIXTURE/launch-$mode.json"
  expect_status 0 "$AZIMUTH" run execute --plan "$FIXTURE/launch-$mode.json" \
    --config "$config" --out "$FIXTURE/bundle-$mode.json"
  expect_status 0 "$AZIMUTH" run verify --bundle "$FIXTURE/bundle-$mode.json"
done
for mode in fault-injected fault-activation fault-inconclusive; do
  config="$FIXTURE/adapters-$mode.json"
  expect_status 0 plan "$FIXTURE/request-fault.json" "$config" \
    --out "$FIXTURE/launch-$mode.json"
  expect_status 0 "$AZIMUTH" run execute --plan "$FIXTURE/launch-$mode.json" \
    --config "$config" --out "$FIXTURE/bundle-$mode.json"
  expect_status 0 "$AZIMUTH" run verify --bundle "$FIXTURE/bundle-$mode.json"
done

# A reidentified raw launch whose asserted form contradicts the selected Challenger is rejected.
expect_status 0 plan "$FIXTURE/request-mutation.json" "$FIXTURE/adapters-killed.json" \
  --out "$FIXTURE/launch-mutation.json"
python3 experiments/challenge-planning/generate.py cross-form "$FIXTURE"
cross_form_output="$FIXTURE/must-not-exist-cross-form.json"
expect_status 1 "$AZIMUTH" run execute --plan "$FIXTURE/launch-cross-form.json" \
  --config "$FIXTURE/adapters-killed.json" --out "$cross_form_output"
test ! -e "$cross_form_output"

# Scheduled work may be omitted only as a partial fact with one scoped diagnostic per omission.
expect_status 0 plan "$FIXTURE/request-omitted.json" "$FIXTURE/adapters-omitted.json" \
  --out "$FIXTURE/launch-omitted.json"
expect_status 0 "$AZIMUTH" run execute --plan "$FIXTURE/launch-omitted.json" \
  --config "$FIXTURE/adapters-omitted.json" --out "$FIXTURE/bundle-omitted.json"
expect_status 0 "$AZIMUTH" run verify --bundle "$FIXTURE/bundle-omitted.json"

# Import uses the same semantic Challenge plan and one exact staged native input.
expect_status 0 plan "$FIXTURE/request-import.json" "$FIXTURE/adapters-killed.json" \
  --out "$FIXTURE/launch-import.json"
expect_status 0 "$AZIMUTH" run import --plan "$FIXTURE/launch-import.json" \
  --input "native=$FIXTURE/native.json" --config "$FIXTURE/adapters-killed.json" \
  --out "$FIXTURE/bundle-import.json"
expect_status 0 "$AZIMUTH" run verify --bundle "$FIXTURE/bundle-import.json"

# A provider selection substitution is exit one and cannot replace a caller-owned sentinel.
expect_status 0 plan "$FIXTURE/request-fault.json" "$FIXTURE/adapters-drift.json" \
  --out "$FIXTURE/launch-drift.json"
printf 'sentinel' >"$FIXTURE/drift-output.json"
expect_status 1 "$AZIMUTH" run execute --plan "$FIXTURE/launch-drift.json" \
  --config "$FIXTURE/adapters-drift.json" --out "$FIXTURE/drift-output.json"
test "$(cat "$FIXTURE/drift-output.json")" = sentinel

python3 - "$FIXTURE" <<'PY'
import hashlib
import json
import pathlib
import re
import sys

root = pathlib.Path(sys.argv[1])
load = lambda name: json.load(open(root / name, encoding="utf-8"))
valid_fp = re.compile(r"sha256:[0-9a-f]{64}").fullmatch

launch = load("launch-all.json")
exported = load("export.json")
assert len(launch["plan"]["challenges"]) == 4
resolutions = {item["plan"]: item for item in exported["challenge_resolutions"]}
selectors = {
    "judgment-claim": (
        ("claim-judgment", "claim", "synthetic#works"),
        ("claim", "synthetic#works"),
        ("claim-judgment", "synthetic#works"),
        ("claim", "synthetic#works"),
    ),
    "judgment-mechanism": (
        ("claim-judgment", "mechanism", "synthetic#guard"),
        ("claim", "synthetic#works"),
        ("claim-judgment", "synthetic#works"),
        ("mechanism", "synthetic#guard"),
    ),
    "judgment-realization": (
        ("claim-judgment", "realization", "core|rust-symbol|synthetic::works"),
        ("claim", "synthetic#works"),
        ("claim-judgment", "synthetic#works"),
        ("realization", "core|rust-symbol|synthetic::works"),
    ),
    "qualification-binding": (
        ("qualification", "binding", "synthetic/edge"),
        ("binding", "synthetic/edge"),
        ("qualification", "synthetic/edge"),
        ("binding", "synthetic/edge"),
    ),
    "qualification-check": (
        ("qualification", "check", "synthetic/analyzer"),
        ("binding", "synthetic/edge"),
        ("qualification", "synthetic/edge"),
        ("check", "synthetic/analyzer"),
    ),
    "qualification-mechanism": (
        ("qualification", "mechanism", "synthetic#guard"),
        ("binding", "synthetic/edge"),
        ("qualification", "synthetic/edge"),
        ("mechanism", "synthetic#guard"),
    ),
    "qualification-realization": (
        ("qualification", "realization", "core|rust-symbol|synthetic::works"),
        ("binding", "synthetic/edge"),
        ("qualification", "synthetic/edge"),
        ("realization", "core|rust-symbol|synthetic::works"),
    ),
}
routes = {
    item["selection"]["id"]: item
    for item in launch["routes"]
    if item["selection"]["kind"] == "challenge"
}
projectable = {
    "check-implementation", "realization", "mechanism-implementation",
    "artifact", "surface-member", "enumeration",
}
for plan, (selector, relation, target, anchor) in selectors.items():
    resolution = resolutions[f"synthetic/{plan}"]
    assert resolution["challenger"] == "mutation/search"
    assert resolution["issues"] == [] and len(resolution["candidates"]) == 1
    candidate = resolution["candidates"][0]
    assert candidate["selector"] == dict(zip(("target", "from", "id"), selector))
    assert candidate["relation"] == dict(zip(("kind", "id"), relation))
    assert (candidate["target"]["kind"], candidate["target"]["id"]) == target
    assert candidate["target"]["expected_fingerprint"] == (
        candidate["target"]["authored_fingerprint"]
    )
    assert valid_fp(candidate["target"]["expected_fingerprint"])
    selection = next(
        item for item in launch["plan"]["challenges"]
        if item["challenger"]["id"] == "mutation/search"
        and (item["target"]["kind"], item["target"]["id"]) == target
    )
    assert any((item["kind"], item["id"]) == anchor for item in selection["scope"]["anchors"])
    route = routes[selection["id"]]
    scoped = {
        (item["kind"], item["id"], item["fingerprint"])
        for item in selection["scope"]["anchors"] + selection["scope"]["inputs"]
        if item["kind"] in projectable
    }
    routed = {
        (item["kind"], item["id"], item["fingerprint"])
        for item in route["inputs"]
    }
    assert routed == scoped and all(valid_fp(item[2]) for item in routed)

source_expected = {
    ("check-implementation", "core|rust-symbol|synthetic::analyze"): {
        "kind": "source", "file": "src/analyzer.rs", "language": "rust",
        "site": "synthetic::analyze",
    },
    ("realization", "core|rust-symbol|synthetic::works"): {
        "kind": "source", "file": "src/synthetic.rs", "language": "rust",
        "site": "synthetic::works",
    },
    ("artifact", "artifact:guard"): {
        "kind": "artifact", "file": "src/schema.sql", "artifact_kind": "sql-index",
        "identity": "core|sql-index|artifact:guard", "unique": True,
        "columns": ["key"], "predicate": None,
    },
}
for route in routes.values():
    for item in route["inputs"]:
        assert item["source"] == source_expected[(item["kind"], item["id"])]

broad = next(
    item for item in launch["plan"]["challenges"]
    if item["challenger"]["id"] == "broad/analyzer"
)
broad_route = routes[broad["id"]]
assert broad["target"]["kind"] == "claim-judgment"
assert {item["kind"] for item in broad["scope"]["inputs"]} == {
    "artifact", "binding", "check", "check-implementation", "claim",
    "claim-judgment", "context", "mechanism", "policy", "qualification",
    "realization",
}
assert broad_route["capability"]["address"] == "synthetic/challenge"
assert broad_route["capability"]["challenge_form"] == "broad-analysis"
fault = next(
    item for item in launch["plan"]["challenges"]
    if item["challenger"]["id"] == "fault/injector"
)
assert ("mechanism", "synthetic#guard") in {
    (item["kind"], item["id"]) for item in fault["scope"]["anchors"]
}
assert routes[fault["id"]]["capability"]["address"] == "synthetic/faults"
assert routes[fault["id"]]["capability"]["challenge_form"] == "fault-injection"

outcomes = {"survivor": "findings", "killed": "clean", "zero": "inconclusive"}
mutation_codes = {
    "survivor": ["mutation/survived"], "killed": [], "zero": ["mutation/no-mutants"],
}
for name, expected in outcomes.items():
    bundle = load(f"bundle-{name}.json")
    assert len(bundle["check_executions"]) == 1
    assert len(bundle["challenger_executions"]) == 1
    check = bundle["check_executions"][0]
    challenge = bundle["challenger_executions"][0]
    assert check["units"][0]["attempts"][0]["activity"] == "shared-analysis"
    assert challenge["units"][0]["attempts"][0]["activity"] == "shared-analysis"
    assert check["observation"]["fingerprint"] != challenge["result"]["fingerprint"]
    assert challenge["target"]["kind"] == "qualification"
    assert challenge["result"]["outcome"] == expected
    assert [item["code"] for item in bundle["diagnostics"]] == mutation_codes[name]

broad_outcomes = {
    "broad-warning": ("findings", ["analyzer/warning"]),
    "broad-clean": ("clean", []),
    "broad-unsupported": ("inconclusive", ["analyzer/unsupported"]),
}
for name, (expected, codes) in broad_outcomes.items():
    bundle = load(f"bundle-{name}.json")
    execution = next(
        item for item in bundle["challenger_executions"]
        if item["challenger"]["id"] == "broad/analyzer"
    )
    mutation = next(
        item for item in bundle["challenger_executions"]
        if item["challenger"]["id"] == "mutation/search"
    )
    assert execution["target"]["kind"] == "claim-judgment"
    assert execution["result"]["outcome"] == expected
    assert mutation["result"]["outcome"] == "clean"
    assert [item["code"] for item in bundle["diagnostics"]] == codes

fault_outcomes = {
    "fault-injected": ("findings", ["fault/injected"]),
    "fault-activation": ("clean", ["fault/activation"]),
    "fault-inconclusive": ("inconclusive", ["fault/inconclusive"]),
}
for name, (expected, codes) in fault_outcomes.items():
    bundle = load(f"bundle-{name}.json")
    execution = next(
        item for item in bundle["challenger_executions"]
        if item["challenger"]["id"] == "fault/injector"
    )
    assert execution["target"]["kind"] == "qualification"
    assert execution["result"]["outcome"] == expected
    assert [item["code"] for item in bundle["diagnostics"]] == codes

activation = load("bundle-fault-activation.json")
check = activation["check_executions"][0]
fault_result = next(
    item for item in activation["challenger_executions"]
    if item["challenger"]["id"] == "fault/injector"
)
assert check["units"][0]["attempts"][0]["activity"] == "shared-analysis"
assert fault_result["units"][0]["attempts"][0]["activity"] == "shared-analysis"
assert check["observation"]["outcome"] == "satisfied"
assert fault_result["result"]["outcome"] == "clean"
assert check["observation"]["fingerprint"] != fault_result["result"]["fingerprint"]

omitted = load("bundle-omitted.json")
assert omitted["status"] == "partial"
scheduled = {
    item["id"] for item in omitted["plan"]["challenges"] if item["lane"] == "scheduled"
}
gate = {
    item["id"] for item in omitted["plan"]["challenges"] if item["lane"] == "gate"
}
actual = {item["id"] for item in omitted["actual_selection"]["challenges"]}
executed = {item["challenge"] for item in omitted["challenger_executions"]}
scoped = {item["scope"]["id"] for item in omitted["diagnostics"]}
assert scheduled and gate and actual == gate and executed == gate and scoped == scheduled

imported = load("bundle-import.json")
native = (root / "native.json").read_bytes()
assert imported["provenance"]["mode"] == "import"
assert imported["provenance"]["adapter"]["import_inputs"] == [{
    "id": "native",
    "digest": "sha256:" + hashlib.sha256(native).hexdigest(),
    "size_bytes": len(native),
}]
for forbidden in ["ledger", "state", "cache", "ingest"]:
    assert not (root / forbidden).exists()
PY

python3 - <<'PY'
import pathlib

for name in [
    "experiments/challenge-planning/generate.py",
    "experiments/challenge-planning/adapters/adapter.py",
    "experiments/challenge-planning/adapters/runtime.py",
]:
    path = pathlib.Path(name)
    compile(path.read_text(encoding="utf-8"), name, "exec")
PY
bash -n experiments/challenge-planning/check.sh
test "$(find experiments/challenge-planning -type f | sort | wc -l | tr -d ' ')" = 5

echo "challenge planning conformance passed"
