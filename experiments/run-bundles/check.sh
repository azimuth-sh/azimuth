#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../.."
REPO_ROOT="$PWD"
EXPERIMENT_TMP="$(mktemp -d "${TMPDIR:-/tmp}/azimuth-run-bundles.XXXXXX")"
trap 'rm -rf "$EXPERIMENT_TMP"' EXIT

python3 experiments/run-bundles/generate.py "$EXPERIMENT_TMP/bundles"
cargo build --quiet --manifest-path tools/azimuth/Cargo.toml
AZIMUTH="$REPO_ROOT/tools/azimuth/target/debug/azimuth"
BUNDLES="$EXPERIMENT_TMP/bundles"

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

VALID=(
  workspace
  ci-candidate
  artifact
  deployment
  service
  monitoring-window
  partial-retry-shards
)

for name in "${VALID[@]}"; do
  expect_status 0 "$AZIMUTH" run verify --bundle "$BUNDLES/$name.json"
  grep -q '^protocol-consistent$' "$EXPERIMENT_TMP/stdout"
done

python3 - "$BUNDLES" <<'PY'
import hashlib
import json
import pathlib
import sys


def fingerprint(value):
    encoded = json.dumps(
        value,
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")
    return "sha256:" + hashlib.sha256(encoded).hexdigest()


root = pathlib.Path(sys.argv[1])
expected = {
    "workspace": "workspace",
    "ci-candidate": "ci-candidate",
    "artifact": "artifact",
    "deployment": "deployment",
    "service": "service",
    "monitoring-window": "monitoring-window",
}
for name, kind in expected.items():
    bundle = json.load(open(root / f"{name}.json", encoding="utf-8"))
    assert bundle["subject"]["kind"] == kind
    provenance = bundle["provenance"]
    adapter = provenance["adapter"]
    assert set(adapter) == {
        "id",
        "adapter_version",
        "adapter_fingerprint",
        "descriptor_fingerprint",
        "configuration_fingerprint",
        "launch_fingerprint",
        "routes",
        "import_inputs",
    }
    assert provenance["normalizer"] == {
        "id": f"adapter/{adapter['id']}",
        "version": adapter["adapter_version"],
        "build_fingerprint": adapter["adapter_fingerprint"],
    }
    assert [route["selection"]["kind"] for route in adapter["routes"]] == [
        "check",
        "challenge",
    ]
    launch_identity = {
        field: adapter[field]
        for field in [
            "id",
            "adapter_version",
            "adapter_fingerprint",
            "descriptor_fingerprint",
            "configuration_fingerprint",
        ]
    }
    assert adapter["launch_fingerprint"] == fingerprint(
        {
            "format": "azimuth-run-launch-fingerprint",
            "version": 1,
            "operation": provenance["mode"],
            "planned_at_ms": bundle["planned_at_ms"],
            "subject": bundle["subject"],
            "subject_fingerprint": bundle["subject_fingerprint"],
            "plan": bundle["plan"],
            "adapter": launch_identity,
            "routes": adapter["routes"],
        }
    )
    assert bundle["run_id"] == fingerprint(
        {
            "format": "azimuth-run-identity",
            "version": 1,
            "source_system": provenance["source"]["system"],
            "source_execution": provenance["source"]["execution"],
            "subject_fingerprint": bundle["subject_fingerprint"],
            "plan_fingerprint": bundle["plan"]["fingerprint"],
            "launch_fingerprint": adapter["launch_fingerprint"],
        }
    )
artifact = json.load(open(root / "artifact.json", encoding="utf-8"))
assert artifact["provenance"]["mode"] == "import"
assert artifact["provenance"]["adapter"]["import_inputs"] == [
    {
        "id": "native-report",
        "digest": "sha256:" + "9" * 64,
        "size_bytes": 37,
    }
]
candidate = json.load(open(root / "ci-candidate.json", encoding="utf-8"))
assert "candidate" not in candidate["subject"]
assert candidate["provenance"]["attributes"]["candidate-ref"]
PY

AGGREGATE=()
for name in "${VALID[@]}"; do
  AGGREGATE+=(--bundle "$BUNDLES/$name.json")
done
expect_status 0 "$AZIMUTH" run verify "${AGGREGATE[@]}"

# One physical shared-probe activity supplies an independent Check attempt and Challenge attempt.
python3 - "$BUNDLES/workspace.json" <<'PY'
import json
import sys

bundle = json.load(open(sys.argv[1], encoding="utf-8"))
check_activities = {
    attempt["activity"]
    for unit in bundle["check_executions"][0]["units"]
    for attempt in unit["attempts"]
}
challenge_activities = {
    attempt["activity"]
    for unit in bundle["challenger_executions"][0]["units"]
    for attempt in unit["attempts"]
}
assert check_activities & challenge_activities == {"shared-probe"}
assert bundle["check_executions"][0]["observation"]["outcome"] == "satisfied"
assert bundle["challenger_executions"][0]["result"]["outcome"] == "clean"
PY

# The partial bundle demonstrates retry recovery inside one selected shard while aggregate
# completeness remains inconclusive. The correction supplies the omitted shard and Challenge.
python3 - "$BUNDLES/partial-retry-shards.json" "$BUNDLES/correction.json" <<'PY'
import json
import sys

partial, correction = [json.load(open(path, encoding="utf-8")) for path in sys.argv[1:]]
attempts = partial["check_executions"][0]["units"][0]["attempts"]
assert [attempt["outcome"] for attempt in attempts] == ["inconclusive", "satisfied"]
assert partial["check_executions"][0]["observation"]["outcome"] == "inconclusive"
assert correction["check_executions"][0]["observation"]["outcome"] == "satisfied"
assert correction["bundle_revision"] == 1
assert correction["corrects"] == partial["bundle_fingerprint"]
assert correction["run_id"] == partial["run_id"]
assert correction["subject"] == partial["subject"]
assert correction["plan"] == partial["plan"]
assert correction["planned_at_ms"] == partial["planned_at_ms"]
assert correction["started_at_ms"] == partial["started_at_ms"]
assert correction["provenance"]["normalizer"] == partial["provenance"]["normalizer"]
for field in [
    "id",
    "adapter_version",
    "adapter_fingerprint",
    "descriptor_fingerprint",
    "configuration_fingerprint",
    "launch_fingerprint",
    "routes",
]:
    assert correction["provenance"]["adapter"][field] == partial["provenance"]["adapter"][field]
PY

expect_status 0 "$AZIMUTH" run verify \
  --bundle "$BUNDLES/partial-retry-shards.json" \
  --bundle "$BUNDLES/partial-retry-shards.json" \
  --bundle "$BUNDLES/correction.json"
expect_status 0 "$AZIMUTH" run verify \
  --bundle "$BUNDLES/correction.json" \
  --bundle "$BUNDLES/partial-retry-shards.json"

expect_status 1 "$AZIMUTH" run verify --bundle "$BUNDLES/mismatch.json"
grep -q 'run/subject-fingerprint' "$EXPERIMENT_TMP/stdout"
expect_status 1 "$AZIMUTH" run inspect \
  --bundle "$BUNDLES/mismatch.json" --format json
python3 - "$EXPERIMENT_TMP/stdout" <<'PY'
import json
import sys

account = json.load(open(sys.argv[1], encoding="utf-8"))
assert account["protocol_consistent"] is False
assert any(item["code"] == "run/subject-fingerprint" for item in account["findings"])
PY

expect_status 2 "$AZIMUTH" run verify --bundle "$BUNDLES/schema-error.json"
test ! -s "$EXPERIMENT_TMP/stdout"
grep -q 'no account was derived' "$EXPERIMENT_TMP/stderr"
expect_status 2 "$AZIMUTH" run inspect --bundle "$BUNDLES/malformed.json" \
  --out "$EXPERIMENT_TMP/must-not-exist"
test ! -e "$EXPERIMENT_TMP/must-not-exist"
test ! -s "$EXPERIMENT_TMP/stdout"

expect_status 0 "$AZIMUTH" run inspect \
  --bundle "$BUNDLES/workspace.json" --format json
python3 - "$EXPERIMENT_TMP/stdout" <<'PY'
import json
import sys

account = json.load(open(sys.argv[1], encoding="utf-8"))
assert account["format"] == "azimuth-run-inspection"
assert account["version"] == 1
assert account["protocol_consistent"] is True
assert account["model_authority"] == "unresolved"
assert account["assurance_state"] == "unresolved"
PY

echo "run bundle conformance passed"
