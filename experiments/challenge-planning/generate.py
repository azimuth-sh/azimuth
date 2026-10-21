#!/usr/bin/env python3
"""Generate strict, self-contained Challenge planning fixtures."""

from __future__ import annotations

import copy
import hashlib
import json
import os
import pathlib
import shutil
import sys


HERE = pathlib.Path(__file__).resolve().parent


def canonical(value: object) -> bytes:
    return json.dumps(value, ensure_ascii=False, separators=(",", ":"), sort_keys=True).encode()


def fingerprint(value: object) -> str:
    return "sha256:" + hashlib.sha256(canonical(value)).hexdigest()


def digest(path: pathlib.Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def fixed(character: str) -> str:
    return "sha256:" + character * 64


def write_json(path: pathlib.Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def configured_adapter(root: pathlib.Path, mode: str, forms=None) -> dict[str, object]:
    forms = forms or ["broad-analysis", "mutation"]
    executable = root / "adapters" / "adapter.py"
    runtime = root / "adapters" / "runtime.py"
    resources = [{"id": "runtime", "digest": digest(runtime)}]
    adapter_fp = fingerprint(
        {
            "format": "azimuth-adapter-fingerprint",
            "version": 1,
            "protocol_version": 1,
            "id": "synthetic",
            "provider_family": "synthetic/challenge-planning",
            "adapter_version": "0.1.0",
            "build": "challenge-planning-1",
            "content": {
                "executable_digest": digest(executable),
                "resources": resources,
            },
        }
    )
    capabilities = [
        {
            "id": "analyze",
            "classes": ["check.execute", "check.import"],
            "challenge_forms": [],
            "semantic_settings": {},
        },
        {
            "id": "challenge",
            "classes": ["challenge.execute", "challenge.import"],
            "challenge_forms": forms,
            "semantic_settings": {},
        },
        {
            "id": "faults",
            "classes": ["challenge.execute", "challenge.import"],
            "challenge_forms": ["fault-injection"],
            "semantic_settings": {},
        },
    ]
    for capability in capabilities:
        capability["fingerprint"] = fingerprint(
            {
                "format": "azimuth-adapter-capability-fingerprint",
                "version": 1,
                "adapter_fingerprint": adapter_fp,
                **capability,
            }
        )
    description = {
        "format": "azimuth-adapter-description",
        "version": 1,
        "protocol_version": 1,
        "id": "synthetic",
        "provider_family": "synthetic/challenge-planning",
        "adapter_version": "0.1.0",
        "build": "challenge-planning-1",
        "content": {
            "executable_digest": digest(executable),
            "resources": resources,
        },
        "adapter_fingerprint": adapter_fp,
        "capabilities": capabilities,
    }
    descriptor_fp = fingerprint(
        {
            "format": "azimuth-adapter-descriptor-fingerprint",
            "version": 1,
            "descriptor": description,
        }
    )
    environment = {"literals": {"MODE": mode, "SYNTHETIC_LITERAL": "exact-value"}}
    limits = {"timeout_ms": 5000, "stdout_bytes": 1000000, "stderr_bytes": 100000}
    configuration_fp = fingerprint(
        {
            "format": "azimuth-adapter-configuration-fingerprint",
            "version": 1,
            "adapter_fingerprint": adapter_fp,
            "descriptor_fingerprint": descriptor_fp,
            "semantic_settings": {},
            "environment": environment,
            "limits": limits,
            "capabilities": capabilities,
        }
    )
    return {
        "id": "synthetic",
        "provider_family": "synthetic/challenge-planning",
        "protocol_version": 1,
        "adapter_version": "0.1.0",
        "build": "challenge-planning-1",
        "content": {
            "executable": {"locator": "adapters/adapter.py", "digest": digest(executable)},
            "resources": [
                {"id": "runtime", "locator": "adapters/runtime.py", "digest": digest(runtime)}
            ],
        },
        "semantic_settings": {},
        "environment": environment,
        "limits": limits,
        "capabilities": capabilities,
        "adapter_fingerprint": adapter_fp,
        "descriptor_fingerprint": descriptor_fp,
        "configuration_fingerprint": configuration_fp,
    }


def verification(
    method_qualification: str,
    applicability: str,
    judgment: str,
    critical_method_qualification: str,
    critical_applicability: str,
    critical_judgment: str,
) -> str:
    plans = [
        ("judgment-requirement", "mutation/search", "requirement-judgment from requirement behavior"),
        (
            "judgment-mechanism",
            "mutation/search",
            "requirement-judgment from mechanism guard",
        ),
        (
            "judgment-realization",
            "mutation/search",
            "requirement-judgment from realization core|rust-symbol|synthetic::works",
        ),
        ("qualification-binding", "mutation/search", "applicability-decision from binding synthetic/edge"),
        ("qualification-check", "mutation/search", "method-qualification from check analyzer"),
        (
            "qualification-mechanism",
            "mutation/search",
            "method-qualification from mechanism guard",
        ),
        (
            "qualification-realization",
            "mutation/search",
            "method-qualification from realization core|rust-symbol|synthetic::works",
        ),
        ("broad-judgment", "broad/analyzer", "requirement-judgment from requirement behavior"),
        (
            "multi-target",
            "mutation/search",
            "applicability-decision from binding synthetic/edge\n"
            "Select: requirement-judgment from requirement behavior",
        ),
        (
            "fault-qualification",
            "fault/injector",
            "applicability-decision from mechanism guard",
        ),
        (
            "critical-judgment",
            "mutation/search",
            "requirement-judgment from requirement critical-behavior",
        ),
        (
            "critical-qualification",
            "mutation/search",
            "applicability-decision from binding synthetic/critical-edge\n"
            "Select: method-qualification from check critical-analyzer",
        ),
    ]
    declarations = []
    for plan_id, verification_challenge, selector in plans:
        declarations.append(
            f"## Challenge Plan: synthetic/{plan_id}\n"
            f"Verification Challenge: {verification_challenge}\nSelect: {selector}\n\n"
            "The authored selector is finite and exact.\n"
        )
    return (
        "# Verification: synthetic\n\n"
        "## Check: analyzer\n"
        "Method: analyze the one synthetic Requirement\n"
        "Terminal: the requirement-specific analyzer returns one bounded fact\n\n"
        "The analyzer is enrolled as a Check.\n\n"
        "## Evidence Binding: synthetic/edge\n"
        "Check: analyzer\nScenario: works\n"
        "Method qualification: synthetic/analyzer-method\nProposition: direct\n"
        "Execution Context: {}\n"
        "Challenge domain: [\"realization\",\"mechanism\"]\nPolicy: credible\n\n"
        "The binding is Scenario-specific.\n\n"
        "## Method Qualification: synthetic/analyzer-method\n"
        "Check: analyzer\nScope: unit\nQuantification: example\nOracle: direct\n"
        "Execution Context: {\"platform\":\"synthetic\"}\n"
        "Challenge domain: [\"realization\",\"mechanism\"]\nPolicy: credible\n"
        "Verdict: qualified\n"
        f"Fingerprint: {method_qualification}\nQualified: 2026-08-22\nQualifier: synthetic-owner\n\n"
        "The exact shared method is current.\n\n"
        "## Applicability Decision: synthetic/edge\nVerdict: applicable\n"
        f"Fingerprint: {applicability}\nDecided: 2026-08-22\nDecider: synthetic-owner\n\n"
        "The qualified method applies to the exact Scenario edge.\n\n"
        "## Requirement Judgment: behavior\nVerdict: accepted\nPolicy: credible\n"
        f"Fingerprint: {judgment}\nJudged: 2026-08-22\nJudge: synthetic-owner\n"
        "Basis: the exact composition is accepted\n"
        "Residual risk: synthetic mutation risk remains\n\n"
        "The total decision is current.\n\n"
        "## Check: critical-analyzer\n"
        "Method: analyze the critical synthetic Requirement\n"
        "Terminal: the critical analyzer returns one bounded fact\n\n"
        "The second analyzer keeps the critical Requirement independent.\n\n"
        "## Evidence Binding: synthetic/critical-edge\n"
        "Check: critical-analyzer\nScenario: critical-works\n"
        "Method qualification: synthetic/critical-analyzer-method\nProposition: direct\n"
        "Execution Context: {}\nChallenge domain: [\"realization\"]\n"
        "Policy: credible\n\nThe critical binding is exact.\n\n"
        "## Method Qualification: synthetic/critical-analyzer-method\n"
        "Check: critical-analyzer\nScope: unit\nQuantification: example\nOracle: direct\n"
        "Execution Context: {\"platform\":\"synthetic\"}\nChallenge domain: [\"realization\"]\n"
        "Policy: credible\nVerdict: qualified\n"
        f"Fingerprint: {critical_method_qualification}\nQualified: 2026-08-22\n"
        "Qualifier: synthetic-owner\n\nThe critical shared method is current.\n\n"
        "## Applicability Decision: synthetic/critical-edge\nVerdict: applicable\n"
        f"Fingerprint: {critical_applicability}\nDecided: 2026-08-22\n"
        "Decider: synthetic-owner\n\nThe critical method applies to the exact Scenario edge.\n\n"
        "## Requirement Judgment: critical-behavior\nVerdict: accepted\nPolicy: credible\n"
        f"Fingerprint: {critical_judgment}\nJudged: 2026-08-22\nJudge: synthetic-owner\n"
        "Basis: the critical composition is accepted\nResidual risk: mutation risk remains\n\n"
        "The critical total decision is current.\n\n"
        "## Verification Challenge: broad/analyzer\nForm: broad-analysis\n"
        "Searches for: broad objections across the Requirement composition\n"
        "Required scope: [\"requirement\",\"realization\",\"artifact\"]\n\n"
        "The analyzer searches more broadly than the enrolled Check.\n\n"
        "## Verification Challenge: fault/injector\nForm: fault-injection\n"
        "Searches for: activation failures under one injected fault\n"
        "Required scope: [\"mechanism\",\"artifact\",\"check-implementation\"]\n\n"
        "The fault injector owns a distinct bounded search.\n\n"
        "## Verification Challenge: mutation/search\nForm: mutation\n"
        "Searches for: mutants that survive the selected behavior\n"
        "Required scope: [\"policy\"]\n\n"
        "The mutation search reports objections rather than evidence.\n\n"
        + "\n".join(declarations)
    )


def request(operation: str, plans: list[str], checks=True, context="synthetic", cap=1):
    return {
        "format": "azimuth-run-plan-request",
        "version": 2,
        "operation": operation,
        "planned_at_ms": 1787300000000,
        "verification_target": {
            "kind": "artifact",
            "artifacts": [{"id": "candidate", "digest": fixed("8")}],
        },
        "required_execution_context": {"platform": context},
        "checks": [
            {
                "id": "analyzer",
                "execution-capability": "synthetic/analyze",
                "scenarios": ["works"],
                "units": [{"id": "whole", "parameters": {}}],
            }
        ] if checks else [],
        "challenges": [
            {
                "id": "synthetic/" + plan,
                "execution-capability": (
                    "synthetic/faults"
                    if plan == "fault-qualification"
                    else "synthetic/challenge"
                ),
                "max_candidates": cap,
                "units": [{"id": "whole", "parameters": {}}],
            }
            for plan in sorted(plans)
        ],
    }


def initialize(root: pathlib.Path) -> None:
    adapters = root / "adapters"
    adapters.mkdir(parents=True)
    for name in ["adapter.py", "runtime.py"]:
        shutil.copyfile(HERE / "adapters" / name, adapters / name)
    os.chmod(adapters / "adapter.py", 0o700)
    for mode in [
        "survivor",
        "killed",
        "zero",
        "broad-warning",
        "broad-clean",
        "broad-unsupported",
        "fault-injected",
        "fault-activation",
        "fault-inconclusive",
        "omitted",
        "drift",
    ]:
        write_json(
            root / f"adapters-{mode}.json",
            {
                "format": "azimuth-adapter-configuration",
                "version": 1,
                "adapters": [configured_adapter(root, mode)],
            },
        )
    write_json(
        root / "adapters-form.json",
        {
            "format": "azimuth-adapter-configuration",
            "version": 1,
            "adapters": [configured_adapter(root, "killed", ["broad-analysis"])],
        },
    )
    package = root / "model" / "synthetic"
    package.mkdir(parents=True)
    (package / "spec.md").write_text(
        "# Spec: synthetic\n\n## Requirement: behavior\nCriticality: standard\n\n"
        "The synthetic unit SHALL preserve its declared behavior.\n\n"
        "### Scenario: works\nThe analyzer and mutation search each emit their own bounded fact.\n\n"
        "## Requirement: critical-behavior\nCriticality: critical\n\n"
        "The critical synthetic unit SHALL preserve its declared behavior.\n\n"
        "### Scenario: critical-works\nThe critical analyzer emits one bounded fact.\n",
        encoding="utf-8",
    )
    (package / "design.md").write_text(
        "# Design: synthetic\n\n## Requirement: behavior\nMechanism: guard\nEnforcement: guard\n"
        "Binding: artifact:guard\nExpect: unique=true\nExpect: columns=key\n\n"
        "The synthetic artifact makes the mechanism addressable.\n\n"
        "## Requirement: critical-behavior\nMechanism: critical-guard\n"
        "Enforcement: constraint\nBinding: artifact:critical-guard\n"
        "Expect: unique=true\nExpect: columns=critical_key\n\n"
        "The critical artifact makes the second mechanism addressable.\n",
        encoding="utf-8",
    )
    (package / "verification.md").write_text(
        verification(fixed("0"), fixed("1"), fixed("2"), fixed("3"), fixed("4"), fixed("5")),
        encoding="utf-8",
    )
    (root / "standards.md").write_text(
        "# Decision policies and Challenge schedule\n\n"
        "## Decision Policy: credible\nRequired challenge: mutation\n\n"
        "Mutation is the minimum current search.\n\n"
        "## Challenge Schedule: current\nGate challenge: mutation\n"
        "Gate challenge: fault-injection\n"
        "Scheduled challenge: broad-analysis\n\nThe two lanes are explicit.\n",
        encoding="utf-8",
    )
    write_json(
        root / "workspace.json",
        {
            "format": "azimuth-workspace",
            "version": 2,
            "areas": [{"id": "core", "mounts": [{"id": "code", "path": "src"}]}],
            "surfaces": [],
            "realization_obligations": [],
        },
    )
    write_json(
        root / "manifest.json",
        {
            "realizes": [
                {
                    "requirement": "behavior",
                    "site": "synthetic::works",
                    "file": "src/synthetic.rs",
                    "lang": "rust",
                    "source_fingerprint": fixed("a"),
                },
                {
                    "requirement": "critical-behavior",
                    "site": "synthetic::critical_works",
                    "file": "src/critical.rs",
                    "lang": "rust",
                    "source_fingerprint": fixed("c"),
                },
            ],
            "check_implementations": [
                {
                    "check": "analyzer",
                    "site": "synthetic::analyze",
                    "file": "src/analyzer.rs",
                    "lang": "rust",
                    "source_fingerprint": fixed("b"),
                },
                {
                    "check": "critical-analyzer",
                    "site": "synthetic::critical_analyze",
                    "file": "src/critical_analyzer.rs",
                    "lang": "rust",
                    "source_fingerprint": fixed("d"),
                },
            ],
            "mechanism_implementations": [],
            "class_members": [],
            "enumerations": [],
            "artifacts": [
                {
                    "id": "artifact:guard",
                    "kind": "sql-index",
                    "file": "src/schema.sql",
                    "unique": True,
                    "columns": ["key"],
                },
                {
                    "id": "artifact:critical-guard",
                    "kind": "sql-index",
                    "file": "src/critical-schema.sql",
                    "unique": True,
                    "columns": ["critical_key"],
                },
            ],
        },
    )
    all_plans = [
        "broad-judgment",
        "fault-qualification",
        "judgment-requirement",
        "judgment-mechanism",
        "judgment-realization",
        "qualification-binding",
        "qualification-check",
        "qualification-mechanism",
        "qualification-realization",
    ]
    write_json(root / "request-all.json", request("execute", all_plans))
    write_json(
        root / "request-fault.json",
        request("execute", ["fault-qualification", "qualification-binding"]),
    )
    write_json(
        root / "request-mutation.json",
        request("execute", ["qualification-binding"]),
    )
    write_json(
        root / "request-analyzers.json",
        request("execute", ["broad-judgment", "judgment-requirement"]),
    )
    write_json(
        root / "request-omitted.json",
        request("execute", ["broad-judgment", "judgment-requirement"], checks=False),
    )
    write_json(
        root / "request-import.json",
        request("import", ["qualification-binding"]),
    )
    write_json(
        root / "request-required-form.json",
        request("execute", ["broad-judgment"], checks=False),
    )
    write_json(
        root / "request-cap.json",
        request("execute", ["multi-target"], checks=False),
    )
    write_json(
        root / "request-context.json",
        request("execute", ["qualification-binding"], checks=False, context="other"),
    )
    write_json(
        root / "request-zero-resolution.json",
        request("execute", ["zero-resolution"], checks=False),
    )
    write_json(root / "native.json", {"source": "synthetic"})


def seal(root: pathlib.Path, export_path: pathlib.Path) -> None:
    exported = json.loads(export_path.read_text(encoding="utf-8"))
    expected = {}
    for collection, kind in [
        ("method_qualifications", "method-qualification"),
        ("applicability_decisions", "applicability-decision"),
        ("requirement_judgments", "requirement-judgment"),
    ]:
        for decision in exported[collection]:
            expected[(kind, decision["id"])] = decision["expected_fingerprint"]
    source_path = root / "model" / "synthetic" / "verification.md"
    source = source_path.read_text(encoding="utf-8")
    source = source.replace(
        fixed("0"), expected[("method-qualification", "synthetic/analyzer-method")]
    )
    source = source.replace(
        fixed("1"), expected[("applicability-decision", "synthetic/edge")]
    )
    source = source.replace(
        fixed("2"), expected[("requirement-judgment", "behavior")]
    )
    source = source.replace(
        fixed("3"), expected[("method-qualification", "synthetic/critical-analyzer-method")]
    )
    source = source.replace(
        fixed("4"), expected[("applicability-decision", "synthetic/critical-edge")]
    )
    source = source.replace(
        fixed("5"), expected[("requirement-judgment", "critical-behavior")]
    )
    source_path.write_text(source, encoding="utf-8")
    negative = root / "model-zero"
    shutil.copytree(root / "model", negative)
    with (negative / "synthetic" / "verification.md").open("a", encoding="utf-8") as handle:
        handle.write(
            "\n## Challenge Plan: synthetic/zero-resolution\n"
            "Verification Challenge: mutation/search\n"
            "Select: requirement-judgment from requirement missing\n\n"
            "This exact authored selector deliberately resolves to zero candidates.\n"
        )


def cross_form(root: pathlib.Path) -> None:
    source = json.loads((root / "launch-mutation.json").read_text(encoding="utf-8"))
    route = next(
        item for item in source["routes"]
        if item["selection"]["kind"] == "challenge"
    )
    route["capability"]["challenge_form"] = "broad-analysis"
    payload = copy.deepcopy(source)
    payload["format"] = "azimuth-run-launch-fingerprint"
    payload.pop("fingerprint")
    payload["adapter"], = payload.pop("adapters")
    source["fingerprint"] = fingerprint(payload)
    write_json(root / "launch-cross-form.json", source)


if len(sys.argv) != 3 or sys.argv[1] not in {"init", "seal", "cross-form"}:
    raise SystemExit(
        "usage: generate.py init <fixture> | seal <fixture> | "
        "cross-form <fixture> (export is <fixture>/initial-export.json)"
    )
if sys.argv[1] == "init":
    initialize(pathlib.Path(sys.argv[2]))
elif sys.argv[1] == "seal":
    fixture = pathlib.Path(sys.argv[2])
    seal(fixture, fixture / "initial-export.json")
else:
    cross_form(pathlib.Path(sys.argv[2]))
