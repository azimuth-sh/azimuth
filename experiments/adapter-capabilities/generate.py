#!/usr/bin/env python3
"""Generate strict, self-contained adapter capability conformance fixtures."""

from __future__ import annotations

import copy
import hashlib
import json
import os
import pathlib
import shutil
import sys


ROOT = pathlib.Path(__file__).resolve().parent
ADAPTER_VERSION = "0.2.0-alpha.2"
BUILD = "adapter-conformance-1"


def canonical(value: object) -> bytes:
    return json.dumps(
        value,
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")


def fingerprint(value: object) -> str:
    return "sha256:" + hashlib.sha256(canonical(value)).hexdigest()


def digest(path: pathlib.Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def fixed(character: str) -> str:
    return "sha256:" + character * 64


def capabilities(role: str) -> list[dict[str, object]]:
    if role == "executor":
        return [
            {
                "id": "checks",
                "classes": ["check.execute"],
                "challenge_forms": [],
                "semantic_settings": {},
            },
            {
                "id": "dual",
                "classes": ["challenge.execute", "check.execute"],
                "challenge_forms": ["fault-injection"],
                "semantic_settings": {},
            },
        ]
    return [
        {
            "id": "reports",
            "classes": ["check.import"],
            "challenge_forms": [],
            "semantic_settings": {},
        }
    ]


def configured_adapter(root: pathlib.Path, role: str, mode: str) -> dict[str, object]:
    adapter_id = role
    provider = "synthetic/execute" if role == "executor" else "synthetic/import"
    executable = root / "adapters" / f"{role}.py"
    runtime = root / "adapters" / "runtime.py"
    resources = [{"id": "runtime", "digest": digest(runtime)}]
    adapter_fingerprint = fingerprint(
        {
            "format": "azimuth-adapter-fingerprint",
            "version": 1,
            "protocol_version": 1,
            "id": adapter_id,
            "provider_family": provider,
            "adapter_version": ADAPTER_VERSION,
            "build": BUILD,
            "content": {
                "executable_digest": digest(executable),
                "resources": resources,
            },
        }
    )
    declared = []
    for item in capabilities(role):
        item = copy.deepcopy(item)
        item["fingerprint"] = fingerprint(
            {
                "format": "azimuth-adapter-capability-fingerprint",
                "version": 1,
                "adapter_fingerprint": adapter_fingerprint,
                **item,
            }
        )
        declared.append(item)
    description = {
        "format": "azimuth-adapter-description",
        "version": 1,
        "protocol_version": 1,
        "id": adapter_id,
        "provider_family": provider,
        "adapter_version": ADAPTER_VERSION,
        "build": BUILD,
        "content": {
            "executable_digest": digest(executable),
            "resources": resources,
        },
        "adapter_fingerprint": adapter_fingerprint,
        "capabilities": declared,
    }
    descriptor_fingerprint = fingerprint(
        {
            "format": "azimuth-adapter-descriptor-fingerprint",
            "version": 1,
            "descriptor": description,
        }
    )
    timeout = 100 if mode == "hang" else 5000
    stdout = 512 if mode == "stdout-overflow" else 1_000_000
    stderr = 512 if mode == "stderr-overflow" else 100_000
    environment = {
        "literals": {
            "MODE": mode,
            "SYNTHETIC_LITERAL": "exact-value",
        }
    }
    limits = {
        "timeout_ms": timeout,
        "stdout_bytes": stdout,
        "stderr_bytes": stderr,
    }
    semantic_settings = {"dialect": "synthetic-v1"}
    configuration_fingerprint = fingerprint(
        {
            "format": "azimuth-adapter-configuration-fingerprint",
            "version": 1,
            "adapter_fingerprint": adapter_fingerprint,
            "descriptor_fingerprint": descriptor_fingerprint,
            "semantic_settings": semantic_settings,
            "environment": environment,
            "limits": limits,
            "capabilities": declared,
        }
    )
    return {
        "id": adapter_id,
        "provider_family": provider,
        "protocol_version": 1,
        "adapter_version": ADAPTER_VERSION,
        "build": BUILD,
        "content": {
            "executable": {
                "locator": f"adapters/{role}.py",
                "digest": digest(executable),
            },
            "resources": [
                {
                    "id": "runtime",
                    "locator": "adapters/runtime.py",
                    "digest": digest(runtime),
                }
            ],
        },
        "semantic_settings": semantic_settings,
        "environment": environment,
        "limits": limits,
        "capabilities": declared,
        "adapter_fingerprint": adapter_fingerprint,
        "descriptor_fingerprint": descriptor_fingerprint,
        "configuration_fingerprint": configuration_fingerprint,
    }


def write_json(path: pathlib.Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def write_configuration(path: pathlib.Path, adapters: list[dict[str, object]]) -> None:
    write_json(
        path,
        {
            "format": "azimuth-adapter-configuration",
            "version": 1,
            "adapters": adapters,
        },
    )


def initialize(root: pathlib.Path) -> None:
    adapters = root / "adapters"
    adapters.mkdir(parents=True)
    for name in ["executor.py", "importer.py", "runtime.py"]:
        shutil.copyfile(ROOT / "adapters" / name, adapters / name)
    for name in ["executor.py", "importer.py"]:
        os.chmod(adapters / name, 0o700)

    normal_executor = configured_adapter(root, "executor", "normal")
    normal_importer = configured_adapter(root, "importer", "normal")
    write_configuration(root / "adapters.json", [normal_executor, normal_importer])
    modes = [
        "cancelled",
        "descriptor-drift",
        "extra-output",
        "hang",
        "malformed",
        "nonzero",
        "partial",
        "schema",
        "stderr-overflow",
        "stdout-overflow",
        "timed-out-fact",
        "violated",
    ]
    for mode in modes:
        write_configuration(
            root / f"adapters-{mode}.json",
            [configured_adapter(root, "executor", mode)],
        )
    drift = json.loads((root / "adapters.json").read_text(encoding="utf-8"))
    drift["adapters"][0]["semantic_settings"]["dialect"] = "unfingerprinted-drift"
    write_json(root / "adapters-configuration-drift.json", drift)

    package = root / "model" / "synthetic"
    package.mkdir(parents=True)
    (package / "spec.md").write_text(
        "# Spec: synthetic/behavior\n\n"
        "## Claim: behavior-is-observable\n"
        "Criticality: standard\n\n"
        "The synthetic component SHALL expose one observable behavior.\n\n"
        "### Case: works\n"
        "The synthetic behavior is observable when checked.\n",
        encoding="utf-8",
    )
    (package / "verification.md").write_text(
        "# Verification: synthetic/behavior\n\n"
        "## Check: synthetic/behavior\n"
        "Method: invoke the synthetic behavior\n"
        "Terminal: one bounded result\n\n"
        "The adapter runs one finite synthetic work unit.\n\n"
        "## Evidence Binding: synthetic/behavior-edge\n"
        "Check: synthetic/behavior\n"
        "Case: synthetic/behavior#behavior-is-observable/works\n"
        "Method qualification: synthetic/behavior-method\n"
        "Proposition: the bounded result directly establishes observable behavior\n"
        "Context: {}\n"
        "Challenge domain: [\"check-implementation\"]\n"
        "Policy: synthetic-evidence\n\n"
        "The experiment binds its one Check to its one Case.\n\n"
        "## Method Qualification: synthetic/behavior-method\n"
        "Check: synthetic/behavior\n"
        "Scope: component\n"
        "Quantification: example\n"
        "Oracle: relational\n"
        "Context: {\"platform\":\"synthetic\"}\n"
        "Challenge domain: [\"check-implementation\"]\n"
        "Policy: synthetic-evidence\n"
        "Verdict: qualified\n"
        "Fingerprint: sha256:2acf632396c94573b0df63d28467ccf16b7cf3d3ffc3fc277e80551789674725\n"
        "Qualified: 2026-08-30\n"
        "Qualifier: adapter-experiment\n\n"
        "The synthetic implementation and oracle are controlled by the fixture.\n\n"
        "## Applicability Decision: synthetic/behavior-edge\n"
        "Verdict: applicable\n"
        "Fingerprint: sha256:15e4326e47b435bbcbbb73f80df45a6f4414c6c706ab16309cb1440cef530ea6\n"
        "Decided: 2026-08-30\n"
        "Decider: adapter-experiment\n\n"
        "The bounded Check applies directly to the fixture Case.\n\n"
        "## Claim Judgment: synthetic/behavior#behavior-is-observable\n"
        "Verdict: accepted\n"
        "Policy: synthetic-evidence\n"
        "Fingerprint: sha256:c1a199c852170cd84bf5699d0b696f98d4c6db95819f2bd380588e0701ac897d\n"
        "Judged: 2026-08-30\n"
        "Judge: adapter-experiment\n"
        "Basis: the current method and binding cover the only Case\n"
        "Residual risk: synthetic execution cannot establish production behavior\n\n"
        "The fixture accepts only its deliberately bounded synthetic composition.\n\n"
        "## Challenger: synthetic/implementation-perturbation\n"
        "Form: implementation-perturbation\n"
        "Searches for: a synthetic implementation change that escapes the bounded result\n"
        "Required scope: [\"policy\"]\n\n"
        "The Challenger attacks the policy basis shared by the synthetic decisions.\n\n"
        "## Challenge Plan: synthetic/decision-coverage\n"
        "Challenger: synthetic/implementation-perturbation\n"
        "Select: method-qualification from check synthetic/behavior\n"
        "Select: applicability-decision from binding synthetic/behavior-edge\n"
        "Select: claim-judgment from claim synthetic/behavior#behavior-is-observable\n\n"
        "The plan covers each positive decision made by the fixture.\n",
        encoding="utf-8",
    )
    (root / "standards.md").write_text(
        "# Decision policies and Challenge schedule\n\n"
        "## Decision Policy: synthetic-evidence\n"
        "Required challenge: implementation-perturbation\n\n"
        "The synthetic fixture requires one bounded implementation challenge form.\n\n"
        "## Challenge Schedule: current\n"
        "Gate challenge: implementation-perturbation\n\n"
        "The one required synthetic form is assigned to the gate lane.\n",
        encoding="utf-8",
    )
    write_json(
        root / "workspace.json",
        {
            "format": "azimuth-workspace",
            "version": 1,
            "areas": [
                {
                    "id": "synthetic",
                    "mounts": [{"id": "code", "path": "src"}],
                }
            ],
            "surfaces": [],
            "realization_obligations": [],
        },
    )
    write_json(
        root / "manifest.json",
        {
            "realizes": [
                {
                    "spec": "synthetic/behavior",
                    "claim": "behavior-is-observable",
                    "site": "behavior::execute",
                    "file": "src/behavior.rs",
                    "lang": "rust-symbol",
                    "source_fingerprint": fixed("9"),
                }
            ],
            "check_implementations": [
                {
                    "check": "synthetic/behavior",
                    "site": "behavior::probe",
                    "file": "src/behavior.rs",
                    "lang": "rust-symbol",
                    "source_fingerprint": fixed("a"),
                }
            ],
            "artifacts": [],
            "class_members": [],
            "enumerations": [],
        },
    )
    subject = {
        "kind": "artifact",
        "artifacts": [{"id": "candidate", "digest": fixed("b")}],
    }
    base = {
        "format": "azimuth-run-plan-request",
        "version": 1,
        "planned_at_ms": 1_787_300_000_000,
        "subject": subject,
        "required_context": {"platform": "synthetic"},
        "checks": [
            {
                "id": "synthetic/behavior",
                "capability": "executor/checks",
                "cases": ["synthetic/behavior#behavior-is-observable/works"],
                "units": [{"id": "whole", "parameters": {}}],
            }
        ],
        "challenges": [],
    }
    execute = {**base, "operation": "execute"}
    imported = copy.deepcopy(base)
    imported["operation"] = "import"
    imported["checks"][0]["capability"] = "importer/reports"
    write_json(root / "request-execute.json", execute)
    write_json(root / "request-import.json", imported)
    (root / "native-report.json").write_text(
        '{"outcome":"satisfied","source":"synthetic"}\n',
        encoding="utf-8",
    )
    relocated = root / "relocated"
    relocated.mkdir()
    shutil.copyfile(root / "native-report.json", relocated / "same-report.json")
    (root / "later-report.json").write_text(
        '{"outcome":"satisfied","source":"synthetic","complete":true}\n',
        encoding="utf-8",
    )


def launch_fingerprint(launch: dict[str, object]) -> str:
    return fingerprint(
        {
            "format": "azimuth-run-launch-fingerprint",
            "version": 1,
            "operation": launch["operation"],
            "planned_at_ms": launch["planned_at_ms"],
            "subject": launch["subject"],
            "subject_fingerprint": launch["subject_fingerprint"],
            "plan": launch["plan"],
            "adapter": launch["adapter"],
            "routes": launch["routes"],
        }
    )


def derive_launches(source: pathlib.Path, config_path: pathlib.Path, target: pathlib.Path) -> None:
    launch = json.loads(source.read_text(encoding="utf-8"))
    configuration = json.loads(config_path.read_text(encoding="utf-8"))
    executor = next(item for item in configuration["adapters"] if item["id"] == "executor")
    dual_capability = next(item for item in executor["capabilities"] if item["id"] == "dual")

    substituted = copy.deepcopy(launch)
    substituted["routes"][0]["capability"] = {
        "address": "executor/dual",
        "class": "check.execute",
        "fingerprint": dual_capability["fingerprint"],
    }
    stale = copy.deepcopy(substituted)
    substituted["fingerprint"] = launch_fingerprint(substituted)
    write_json(target / "substitution-stale.json", stale)
    write_json(target / "substitution-reidentified.json", substituted)

    dual = copy.deepcopy(substituted)
    challenger_fingerprint = fixed("c")
    target_fingerprint = fixed("d")
    anchors = [
        {
            "kind": "claim",
            "id": "synthetic/behavior#works",
            "fingerprint": fixed("e"),
        }
    ]
    inputs = [
        {
            "kind": "claim",
            "id": "synthetic/behavior#works",
            "fingerprint": fixed("e"),
        },
        {
            "kind": "claim-judgment",
            "id": "synthetic/behavior#works",
            "fingerprint": target_fingerprint,
        },
        {
            "kind": "realization",
            "id": "synthetic|rust-symbol|behavior::execute",
            "fingerprint": fixed("f"),
        },
        {
            "kind": "policy",
            "id": "synthetic/claim-judgment-policy",
            "fingerprint": fixed("9"),
        },
    ]
    scope_fingerprint = fingerprint(
        {
            "format": "azimuth-challenge-scope-fingerprint",
            "version": 1,
            "anchors": anchors,
            "inputs": inputs,
        }
    )
    selection_fingerprint = fingerprint(
        {
            "format": "azimuth-challenge-selection-identity",
            "version": 1,
            "challenger_fingerprint": challenger_fingerprint,
            "target_kind": "claim-judgment",
            "target_fingerprint": target_fingerprint,
        }
    )
    challenge_id = "challenge/" + selection_fingerprint.removeprefix("sha256:")
    challenge = {
        "id": challenge_id,
        "challenger": {
            "id": "fault-injection/synthetic",
            "fingerprint": challenger_fingerprint,
        },
        "target": {
            "kind": "claim-judgment",
            "id": "synthetic/behavior#works",
            "fingerprint": target_fingerprint,
        },
        "lane": "gate",
        "scope": {
            "anchors": anchors,
            "inputs": inputs,
            "fingerprint": scope_fingerprint,
        },
        "units": [{"id": "whole", "parameters": {}}],
    }
    dual["plan"]["challenges"] = [challenge]
    dual["plan"]["fingerprint"] = fingerprint(
        {
            "format": "azimuth-run-plan-fingerprint",
            "version": 1,
            "subject_fingerprint": dual["subject_fingerprint"],
            "model_fingerprint": dual["plan"]["model_fingerprint"],
            "required_context": dual["plan"]["required_context"],
            "checks": dual["plan"]["checks"],
            "challenges": dual["plan"]["challenges"],
        }
    )
    dual["routes"].append(
        {
            "selection": {"kind": "challenge", "id": challenge_id},
            "capability": {
                "address": "executor/dual",
                "class": "challenge.execute",
                "challenge_form": "fault-injection",
                "fingerprint": dual_capability["fingerprint"],
            },
            "inputs": [
                {
                    "kind": "realization",
                    "id": "synthetic|rust-symbol|behavior::execute",
                    "fingerprint": fixed("f"),
                    "source": {
                        "kind": "source",
                        "file": "src/behavior.rs",
                        "language": "rust-symbol",
                        "site": "behavior::execute",
                    },
                }
            ],
        }
    )
    dual["fingerprint"] = launch_fingerprint(dual)
    write_json(target / "dual-role.json", dual)


def request_fingerprint(
    operation: str,
    launch_fingerprint: str,
    inputs: list[dict[str, object]],
    predecessors: list[dict[str, object]],
) -> str:
    return fingerprint(
        {
            "format": "azimuth-adapter-request-fingerprint",
            "version": 1,
            "operation": operation,
            "launch_fingerprint": launch_fingerprint,
            "inputs": inputs,
            "predecessors": predecessors,
        }
    )


def invalid_requests(fixture: pathlib.Path, runtime: pathlib.Path) -> None:
    launch = json.loads((fixture / "launch-import.json").read_text(encoding="utf-8"))
    terminal = json.loads((fixture / "imported.json").read_text(encoding="utf-8"))
    native_path = (fixture / "native-report.json").resolve()
    native_content = native_path.read_bytes()
    identities = [
        {
            "id": "native-report",
            "digest": "sha256:" + hashlib.sha256(native_content).hexdigest(),
            "size_bytes": len(native_content),
        }
    ]
    inputs = [{**identities[0], "locator": str(native_path)}]
    predecessor = {
        "bundle_revision": terminal["bundle_revision"],
        "bundle_fingerprint": terminal["bundle_fingerprint"],
    }
    base = {
        "format": "azimuth-adapter-request",
        "version": 1,
        "request_id": request_fingerprint(
            "import",
            launch["fingerprint"],
            identities,
            [predecessor],
        ),
        "operation": "import",
        "launch_plan": launch,
        "configuration": {
            "fingerprint": launch["adapter"]["configuration_fingerprint"],
            "semantic_settings": {"dialect": "synthetic-v1"},
            "resources": [
                {
                    "id": "runtime",
                    "digest": digest(runtime),
                    "locator": str(runtime.resolve()),
                }
            ],
            "capabilities": [],
        },
        "inputs": inputs,
        "predecessors": [predecessor],
        "terminal_predecessor": terminal,
    }
    absent = copy.deepcopy(base)
    absent.pop("predecessors")
    write_json(fixture / "request-predecessors-absent.json", absent)

    malformed = copy.deepcopy(base)
    malformed["predecessors"][0]["bundle_revision"] = 7
    write_json(fixture / "request-predecessors-malformed.json", malformed)

    stale = copy.deepcopy(base)
    stale["request_id"] = request_fingerprint(
        "import",
        launch["fingerprint"],
        identities,
        [],
    )
    write_json(fixture / "request-predecessor-identity-stale.json", stale)


def main() -> None:
    if len(sys.argv) < 3:
        raise SystemExit(
            "usage: generate.py init <dir> | launches <launch> <config> <dir> | "
            "invalid-requests <fixture> <runtime>"
        )
    if sys.argv[1] == "init" and len(sys.argv) == 3:
        initialize(pathlib.Path(sys.argv[2]).resolve())
        return
    if sys.argv[1] == "launches" and len(sys.argv) == 5:
        derive_launches(
            pathlib.Path(sys.argv[2]),
            pathlib.Path(sys.argv[3]),
            pathlib.Path(sys.argv[4]),
        )
        return
    if sys.argv[1] == "invalid-requests" and len(sys.argv) == 4:
        invalid_requests(pathlib.Path(sys.argv[2]), pathlib.Path(sys.argv[3]))
        return
    raise SystemExit("invalid generate.py arguments")


if __name__ == "__main__":
    main()
