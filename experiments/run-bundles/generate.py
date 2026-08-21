#!/usr/bin/env python3
"""Generate self-contained strict Run bundle conformance fixtures."""

from __future__ import annotations

import copy
import hashlib
import json
import pathlib
import sys


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
        "id": "synthetic/recovery-check",
        "fingerprint": fixed("b"),
        "implementations": [
            {
                "identity": "synthetic|rust-symbol|recovery::probe",
                "source_fingerprint": fixed("c"),
            }
        ],
        "units": copy.deepcopy(units),
    }


def challenge_selection(units: list[dict[str, object]]) -> dict[str, object]:
    return {
        "id": "synthetic-credibility",
        "challenger": {
            "id": "mutation/implementation-perturbation",
            "fingerprint": fixed("d"),
        },
        "target": {
            "kind": "qualification",
            "id": "synthetic/recovery-edge",
            "fingerprint": fixed("e"),
        },
        "units": copy.deepcopy(units),
    }


def subject(kind: str) -> dict[str, object]:
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


def base_bundle(
    label: str,
    subject_kind: str,
    provenance_mode: str = "execute",
) -> dict[str, object]:
    check_units = [unit("shard-a"), unit("shard-b")]
    challenge_units = [unit("whole")]
    check = check_selection(check_units)
    challenge = challenge_selection(challenge_units)
    bundle: dict[str, object] = {
        "format": "azimuth-run-bundle",
        "version": 1,
        "run_id": fixed("0"),
        "bundle_revision": 0,
        "bundle_fingerprint": fixed("0"),
        "subject": subject(subject_kind),
        "subject_fingerprint": fixed("0"),
        "planned_at_ms": 100,
        "started_at_ms": 110,
        "finished_at_ms": 200,
        "status": "complete",
        "plan": {
            "model_fingerprint": fixed("a"),
            "required_context": {"platform": "synthetic"},
            "checks": [copy.deepcopy(check)],
            "challenges": [copy.deepcopy(challenge)],
            "fingerprint": fixed("0"),
        },
        "actual_selection": {
            "context": {"platform": "synthetic"},
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
            "normalizer": {"id": "azimuth/local", "version": "alpha.2"},
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
                                "outcome": "inconclusive",
                            },
                            {
                                "ordinal": 2,
                                "activity": "shared-probe",
                                "outcome": "satisfied",
                            },
                        ],
                    },
                    {
                        "id": "shard-b",
                        "attempts": [
                            {
                                "ordinal": 1,
                                "activity": "shard-b-probe",
                                "outcome": "satisfied",
                            }
                        ],
                    },
                ],
                "observation": {
                    "outcome": "satisfied",
                    "observed_at_ms": 160,
                    "fingerprint": fixed("0"),
                    "artifacts": [],
                    "diagnostics": [],
                },
            }
        ],
        "challenger_executions": [
            {
                "challenge": challenge["id"],
                "challenger": copy.deepcopy(challenge["challenger"]),
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
    if subject_kind == "ci-candidate":
        bundle["provenance"]["attributes"] = {"candidate-ref": "refs/pull/17/head"}
    return refresh(bundle)


def refresh(bundle: dict[str, object]) -> dict[str, object]:
    bundle["subject_fingerprint"] = fingerprint(
        {
            "format": "azimuth-subject-fingerprint",
            "version": 1,
            "subject": bundle["subject"],
        }
    )
    plan = bundle["plan"]
    plan["fingerprint"] = fingerprint(
        {
            "format": "azimuth-run-plan-fingerprint",
            "version": 1,
            "subject_fingerprint": bundle["subject_fingerprint"],
            "model_fingerprint": plan["model_fingerprint"],
            "required_context": plan["required_context"],
            "checks": plan["checks"],
            "challenges": plan["challenges"],
        }
    )
    selection = bundle["actual_selection"]
    selection["plan_fingerprint"] = plan["fingerprint"]
    selection["fingerprint"] = fingerprint(
        {
            "format": "azimuth-run-selection-fingerprint",
            "version": 1,
            "plan_fingerprint": selection["plan_fingerprint"],
            "context": selection["context"],
            "checks": selection["checks"],
            "challenges": selection["challenges"],
        }
    )
    source = bundle["provenance"]["source"]
    bundle["run_id"] = fingerprint(
        {
            "format": "azimuth-run-identity",
            "version": 1,
            "source_system": source["system"],
            "source_execution": source["execution"],
            "subject_fingerprint": bundle["subject_fingerprint"],
            "plan_fingerprint": plan["fingerprint"],
        }
    )
    for execution in bundle["check_executions"]:
        observation = execution["observation"]
        observation["fingerprint"] = fingerprint(
            {
                "format": "azimuth-observation-fingerprint",
                "version": 1,
                "run_id": bundle["run_id"],
                "subject_fingerprint": bundle["subject_fingerprint"],
                "check": execution["check"],
                "context": selection["context"],
                "outcome": observation["outcome"],
                "observed_at_ms": observation["observed_at_ms"],
            }
        )
    for execution in bundle["challenger_executions"]:
        result = execution["result"]
        result["fingerprint"] = fingerprint(
            {
                "format": "azimuth-challenge-result-fingerprint",
                "version": 1,
                "run_id": bundle["run_id"],
                "challenge": execution["challenge"],
                "challenger": execution["challenger"],
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
            "version": 1,
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
    bundle["check_executions"][0]["observation"]["outcome"] = "inconclusive"
    bundle["check_executions"][0]["observation"]["observed_at_ms"] = 140
    bundle["challenger_executions"] = []
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
    mismatch["subject_fingerprint"] = fixed("9")
    write(output / "mismatch.json", mismatch)

    schema_error = copy.deepcopy(base_bundle("schema-error", "workspace"))
    schema_error["provider"] = "forbidden"
    write(output / "schema-error.json", schema_error)
    (output / "malformed.json").write_text("{\n", encoding="utf-8")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("usage: generate.py <output-directory>")
    generate(pathlib.Path(sys.argv[1]))
