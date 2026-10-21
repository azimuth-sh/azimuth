#!/usr/bin/env python3
"""Generate self-contained strict Run bundle conformance fixtures."""

from __future__ import annotations

import copy
import hashlib
import json
import pathlib
import sys


ADAPTER_ID = "synthetic"
ADAPTER_VERSION = "0.1.0-alpha.2"
ADAPTER_FINGERPRINT = "sha256:" + "4" * 64
DESCRIPTOR_FINGERPRINT = "sha256:" + "5" * 64
CONFIGURATION_FINGERPRINT = "sha256:" + "6" * 64
CHECK_CAPABILITY_FINGERPRINT = "sha256:" + "7" * 64
CHALLENGE_CAPABILITY_FINGERPRINT = "sha256:" + "8" * 64
CHECK_SCENARIO = "recovery-works"


def fingerprint(value: object) -> str:
    encoded = json.dumps(
        value,
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")
    return "sha256:" + hashlib.sha256(encoded).hexdigest()


def fixed(seed: str) -> str:
    return "sha256:" + seed * 64


def unit(unit_id: str) -> dict[str, object]:
    return {"id": unit_id, "parameters": {"shard": unit_id}}


def check_selection(units: list[dict[str, object]]) -> dict[str, object]:
    return {
        "id": "recovery-check",
        "fingerprint": fixed("b"),
        "scenarios": [CHECK_SCENARIO],
        "implementations": [
            {
                "identity": "synthetic|rust-symbol|recovery::probe",
                "source_fingerprint": fixed("c"),
            }
        ],
        "units": copy.deepcopy(units),
    }


def challenge_selection(units: list[dict[str, object]]) -> dict[str, object]:
    verification_challenge_fingerprint = fixed("d")
    target_fingerprint = fixed("e")
    anchors = [
        {
            "kind": "realization",
            "id": "synthetic|rust-symbol|recovery::execute",
            "fingerprint": fixed("f"),
        }
    ]
    inputs = [
        {
            "kind": "requirement",
            "id": "recovers",
            "fingerprint": fixed("1"),
        },
        {
            "kind": "binding",
            "id": "synthetic/recovery-edge",
            "fingerprint": fixed("2"),
        },
        {
            "kind": "method-qualification",
            "id": "synthetic/recovery-edge",
            "fingerprint": target_fingerprint,
        },
        {
            "kind": "check",
            "id": "recovery-check",
            "fingerprint": fixed("b"),
        },
        {
            "kind": "check-implementation",
            "id": "synthetic|rust-symbol|recovery::probe",
            "fingerprint": fixed("c"),
        },
        {
            "kind": "context",
            "id": "synthetic/recovery-edge",
            "fingerprint": fixed("3"),
        },
        {
            "kind": "policy",
            "id": "synthetic/recovery-policy",
            "fingerprint": fixed("9"),
        },
    ]
    scope_fingerprint = fingerprint(
        {
            "format": "azimuth-challenge-scope-fingerprint",
            "version": 2,
            "anchors": anchors,
            "inputs": inputs,
        }
    )
    selection_fingerprint = fingerprint(
        {
            "format": "azimuth-challenge-selection-identity",
            "version": 2,
            "verification_challenge_fingerprint": verification_challenge_fingerprint,
            "target_kind": "method-qualification",
            "target_fingerprint": target_fingerprint,
        }
    )
    return {
        "id": "challenge/" + selection_fingerprint.removeprefix("sha256:"),
        "verification_challenge": {
            "id": "mutation/implementation-perturbation",
            "fingerprint": verification_challenge_fingerprint,
        },
        "target": {
            "kind": "method-qualification",
            "id": "synthetic/recovery-edge",
            "fingerprint": target_fingerprint,
        },
        "lane": "gate",
        "scope": {
            "anchors": anchors,
            "inputs": inputs,
            "fingerprint": scope_fingerprint,
        },
        "units": copy.deepcopy(units),
    }


def challenge_launch_inputs() -> list[dict[str, object]]:
    return [
        {
            "kind": "check-implementation",
            "id": "synthetic|rust-symbol|recovery::probe",
            "fingerprint": fixed("c"),
            "source": {
                "kind": "source",
                "file": "tests/recovery.rs",
                "language": "rust",
                "site": "recovery::probe",
            },
        },
        {
            "kind": "realization",
            "id": "synthetic|rust-symbol|recovery::execute",
            "fingerprint": fixed("f"),
            "source": {
                "kind": "source",
                "file": "src/recovery.rs",
                "language": "rust",
                "site": "recovery::execute",
            },
        },
    ]


def verification_target(kind: str) -> dict[str, object]:
    repository = {
        "id": "root",
        "revision": "synthetic-revision",
        "content_fingerprint": fixed("1"),
    }
    artifact = {"id": "synthetic-image", "digest": fixed("2")}
    if kind == "workspace":
        return {"kind": kind, "repositories": [repository]}
    if kind == "ci-candidate":
        return {"kind": kind, "repositories": [repository]}
    if kind == "artifact":
        return {"kind": kind, "artifacts": [artifact]}
    if kind == "deployment":
        return {
            "kind": kind,
            "environment": "staging",
            "deployment": "synthetic/release-17",
            "deployment_fingerprint": fixed("3"),
            "artifacts": [artifact],
        }
    if kind == "service":
        return {
            "kind": kind,
            "environment": "staging",
            "service": "synthetic/api",
            "deployment": "synthetic/release-17",
            "deployment_fingerprint": fixed("3"),
        }
    if kind == "monitoring-window":
        return {
            "kind": kind,
            "environment": "staging",
            "services": [
                {
                    "service": "synthetic/api",
                    "deployment": "synthetic/release-17",
                    "deployment_fingerprint": fixed("3"),
                }
            ],
            "window_start_ms": 120,
            "window_end_ms": 180,
        }
    raise ValueError(kind)


def activity(
    activity_id: str,
    status: str,
    started: int,
    finished: int,
) -> dict[str, object]:
    return {
        "id": activity_id,
        "status": status,
        "started_at_ms": started,
        "finished_at_ms": finished,
        "artifacts": [],
        "diagnostics": [],
        "attributes": {},
    }


def routes(plan: dict[str, object], mode: str) -> list[dict[str, object]]:
    result = []
    for check in plan["checks"]:
        result.append(
            {
                "selection": {"kind": "check", "id": check["id"]},
                "capability": {
                    "address": f"{ADAPTER_ID}/checks",
                    "class": f"check.{mode}",
                    "fingerprint": CHECK_CAPABILITY_FINGERPRINT,
                },
            }
        )
    for challenge in plan["challenges"]:
        result.append(
            {
                "selection": {"kind": "challenge", "id": challenge["id"]},
                "capability": {
                    "address": f"{ADAPTER_ID}/challenges",
                    "class": f"challenge.{mode}",
                    "challenge_form": "implementation-perturbation",
                    "fingerprint": CHALLENGE_CAPABILITY_FINGERPRINT,
                },
                "inputs": challenge_launch_inputs(),
            }
        )
    return result


def import_inputs(mode: str) -> list[dict[str, object]]:
    if mode == "execute":
        return []
    return [{"id": "native-report", "digest": fixed("9"), "size_bytes": 37}]


def adapter_identity() -> dict[str, object]:
    return {
        "id": ADAPTER_ID,
        "adapter_version": ADAPTER_VERSION,
        "adapter_fingerprint": ADAPTER_FINGERPRINT,
        "descriptor_fingerprint": DESCRIPTOR_FINGERPRINT,
        "configuration_fingerprint": CONFIGURATION_FINGERPRINT,
    }


def base_bundle(
    label: str,
    target_kind: str,
    provenance_mode: str = "execute",
) -> dict[str, object]:
    check_units = [unit("shard-a"), unit("shard-b")]
    challenge_units = [unit("whole")]
    check = check_selection(check_units)
    challenge = challenge_selection(challenge_units)
    bundle: dict[str, object] = {
        "format": "azimuth-run-bundle",
        "version": 2,
        "run_id": fixed("0"),
        "bundle_revision": 0,
        "bundle_fingerprint": fixed("0"),
        "verification_target": verification_target(target_kind),
        "verification_target_fingerprint": fixed("0"),
        "planned_at_ms": 100,
        "started_at_ms": 110,
        "finished_at_ms": 200,
        "status": "complete",
        "plan": {
            "model_fingerprint": fixed("a"),
            "required_execution_context": {"platform": "synthetic"},
            "checks": [copy.deepcopy(check)],
            "challenges": [copy.deepcopy(challenge)],
            "fingerprint": fixed("0"),
        },
        "actual_selection": {
            "execution_context": {"platform": "synthetic"},
            "plan_fingerprint": fixed("0"),
            "checks": [copy.deepcopy(check)],
            "challenges": [copy.deepcopy(challenge)],
            "fingerprint": fixed("0"),
        },
        "provenance": {
            "mode": provenance_mode,
            "source": {
                "system": "synthetic-runner",
                "execution": f"conformance/{label}",
            },
            "normalizer": {
                "id": f"adapter/{ADAPTER_ID}",
                "version": ADAPTER_VERSION,
                "build_fingerprint": ADAPTER_FINGERPRINT,
            },
            "adapter": {
                **adapter_identity(),
                "launch_fingerprint": fixed("0"),
                "routes": routes(
                    {
                        "checks": [check],
                        "challenges": [challenge],
                    },
                    provenance_mode,
                ),
                "import_inputs": import_inputs(provenance_mode),
            },
            "generated_at_ms": 210,
        },
        "artifacts": [],
        "diagnostics": [],
        "activities": [
            activity("retry-timeout", "timed-out", 115, 120),
            activity("shard-b-probe", "completed", 141, 160),
            activity("shared-probe", "completed", 121, 140),
        ],
        "check_executions": [
            {
                "check": {
                    "id": check["id"],
                    "fingerprint": check["fingerprint"],
                },
                "units": [
                    {
                        "id": "shard-a",
                        "attempts": [
                            {
                                "ordinal": 1,
                                "activity": "retry-timeout",
                                "outcomes": {CHECK_SCENARIO: "inconclusive"},
                            },
                            {
                                "ordinal": 2,
                                "activity": "shared-probe",
                                "outcomes": {CHECK_SCENARIO: "satisfied"},
                            },
                        ],
                    },
                    {
                        "id": "shard-b",
                        "attempts": [
                            {
                                "ordinal": 1,
                                "activity": "shard-b-probe",
                                "outcomes": {CHECK_SCENARIO: "satisfied"},
                            }
                        ],
                    },
                ],
                "results": [
                    {
                        "scenario": CHECK_SCENARIO,
                        "outcome": "satisfied",
                        "observed_at_ms": 160,
                        "fingerprint": fixed("0"),
                        "artifacts": [],
                        "diagnostics": [],
                    }
                ],
            }
        ],
        "verification_challenge_executions": [
            {
                "challenge": challenge["id"],
                "verification_challenge": copy.deepcopy(challenge["verification_challenge"]),
                "target": copy.deepcopy(challenge["target"]),
                "units": [
                    {
                        "id": "whole",
                        "attempts": [
                            {
                                "ordinal": 1,
                                "activity": "shared-probe",
                                "outcome": "clean",
                            }
                        ],
                    }
                ],
                "result": {
                    "outcome": "clean",
                    "observed_at_ms": 160,
                    "fingerprint": fixed("0"),
                    "objections": [],
                    "artifacts": [],
                    "diagnostics": [],
                },
            }
        ],
    }
    if target_kind == "ci-candidate":
        bundle["provenance"]["attributes"] = {"candidate-ref": "refs/pull/17/head"}
    return refresh(bundle)


def refresh(bundle: dict[str, object]) -> dict[str, object]:
    bundle["verification_target_fingerprint"] = fingerprint(
        {
            "format": "azimuth-verification-target-fingerprint",
            "version": 2,
            "verification_target": bundle["verification_target"],
        }
    )
    plan = bundle["plan"]
    plan["fingerprint"] = fingerprint(
        {
            "format": "azimuth-run-plan-fingerprint",
            "version": 2,
            "verification_target_fingerprint": bundle["verification_target_fingerprint"],
            "model_fingerprint": plan["model_fingerprint"],
            "required_execution_context": plan["required_execution_context"],
            "checks": plan["checks"],
            "challenges": plan["challenges"],
        }
    )
    selection = bundle["actual_selection"]
    selection["plan_fingerprint"] = plan["fingerprint"]
    selection["fingerprint"] = fingerprint(
        {
            "format": "azimuth-run-selection-fingerprint",
            "version": 2,
            "plan_fingerprint": selection["plan_fingerprint"],
            "execution_context": selection["execution_context"],
            "checks": selection["checks"],
            "challenges": selection["challenges"],
        }
    )
    provenance = bundle["provenance"]
    adapter = provenance["adapter"]
    adapter["launch_fingerprint"] = fingerprint(
        {
            "format": "azimuth-run-launch-fingerprint",
            "version": 2,
            "operation": provenance["mode"],
            "planned_at_ms": bundle["planned_at_ms"],
            "verification_target": bundle["verification_target"],
            "verification_target_fingerprint": bundle["verification_target_fingerprint"],
            "plan": plan,
            "adapter": adapter_identity(),
            "routes": adapter["routes"],
        }
    )
    source = bundle["provenance"]["source"]
    bundle["run_id"] = fingerprint(
        {
            "format": "azimuth-run-identity",
            "version": 2,
            "source_system": source["system"],
            "source_execution": source["execution"],
            "verification_target_fingerprint": bundle["verification_target_fingerprint"],
            "plan_fingerprint": plan["fingerprint"],
            "launch_fingerprint": adapter["launch_fingerprint"],
        }
    )
    for execution in bundle["check_executions"]:
        for check_result in execution["results"]:
            check_result["fingerprint"] = fingerprint(
                {
                    "format": "azimuth-check-result-fingerprint",
                    "version": 2,
                    "run_id": bundle["run_id"],
                    "verification_target_fingerprint": bundle["verification_target_fingerprint"],
                    "check": execution["check"],
                    "scenario": check_result["scenario"],
                    "execution_context": selection["execution_context"],
                    "outcome": check_result["outcome"],
                    "observed_at_ms": check_result["observed_at_ms"],
                }
            )
    for execution in bundle["verification_challenge_executions"]:
        result = execution["result"]
        result["fingerprint"] = fingerprint(
            {
                "format": "azimuth-challenge-result-fingerprint",
                "version": 2,
                "run_id": bundle["run_id"],
                "challenge": execution["challenge"],
                "verification_challenge": execution["verification_challenge"],
                "target": execution["target"],
                "outcome": result["outcome"],
                "observed_at_ms": result["observed_at_ms"],
            }
        )
    without_fingerprint = copy.deepcopy(bundle)
    without_fingerprint.pop("bundle_fingerprint")
    bundle["bundle_fingerprint"] = fingerprint(
        {
            "format": "azimuth-run-bundle-fingerprint",
            "version": 2,
            "bundle": without_fingerprint,
        }
    )
    return bundle


def partial_bundle(label: str) -> dict[str, object]:
    bundle = base_bundle(label, "workspace")
    bundle["status"] = "partial"
    bundle["actual_selection"]["checks"][0]["units"] = [unit("shard-a")]
    bundle["actual_selection"]["challenges"] = []
    bundle["activities"] = [
        item
        for item in bundle["activities"]
        if item["id"] in {"retry-timeout", "shared-probe"}
    ]
    bundle["check_executions"][0]["units"] = bundle["check_executions"][0]["units"][:1]
    bundle["check_executions"][0]["results"][0]["outcome"] = "inconclusive"
    bundle["check_executions"][0]["results"][0]["observed_at_ms"] = 140
    bundle["verification_challenge_executions"] = []
    challenge_id = bundle["plan"]["challenges"][0]["id"]
    bundle["diagnostics"] = [
        {
            "id": "challenge/execution-omission",
            "class": "execution",
            "severity": "warning",
            "code": "challenge/not-executed",
            "message": "The planned Challenge did not execute before this partial result.",
            "scope": {"kind": "challenge-selection", "id": challenge_id},
            "artifacts": [],
            "details": {"reason": "partial-run"},
        }
    ]
    return refresh(bundle)


def write(path: pathlib.Path, bundle: dict[str, object]) -> None:
    path.write_text(json.dumps(bundle, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def generate(output: pathlib.Path) -> None:
    output.mkdir(parents=True, exist_ok=True)
    for kind in [
        "workspace",
        "ci-candidate",
        "artifact",
        "deployment",
        "service",
        "monitoring-window",
    ]:
        mode = "import" if kind == "artifact" else "execute"
        write(output / f"{kind}.json", base_bundle(kind, kind, mode))

    partial = partial_bundle("correction-chain")
    write(output / "partial-retry-shards.json", partial)
    correction = base_bundle("correction-chain", "workspace")
    correction["bundle_revision"] = 1
    correction["corrects"] = partial["bundle_fingerprint"]
    correction["correction_reason"] = "the second shard and challenge completed"
    correction["finished_at_ms"] = 220
    correction["provenance"]["generated_at_ms"] = 230
    refresh(correction)
    write(output / "correction.json", correction)

    mismatch = copy.deepcopy(base_bundle("mismatch", "service"))
    mismatch["verification_target_fingerprint"] = fixed("9")
    write(output / "mismatch.json", mismatch)

    schema_error = copy.deepcopy(base_bundle("schema-error", "workspace"))
    schema_error["provider"] = "forbidden"
    write(output / "schema-error.json", schema_error)
    (output / "malformed.json").write_text("{\n", encoding="utf-8")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("usage: generate.py <output-directory>")
    generate(pathlib.Path(sys.argv[1]))
