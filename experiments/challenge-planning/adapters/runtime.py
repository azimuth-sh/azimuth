"""Standard-library runtime for Challenge planning conformance."""

import copy
import hashlib
import json
import os
import sys


def canonical(value):
    return json.dumps(value, ensure_ascii=False, separators=(",", ":"), sort_keys=True)


def fingerprint(value):
    return "sha256:" + hashlib.sha256(canonical(value).encode("utf-8")).hexdigest()


CAPABILITIES = [
    {
        "id": "analyze",
        "classes": ["check.execute", "check.import"],
        "challenge_forms": [],
        "semantic_settings": {},
    },
    {
        "id": "challenge",
        "classes": ["challenge.execute", "challenge.import"],
        "challenge_forms": ["broad-analysis", "mutation"],
        "semantic_settings": {},
    },
    {
        "id": "faults",
        "classes": ["challenge.execute", "challenge.import"],
        "challenge_forms": ["fault-injection"],
        "semantic_settings": {},
    },
]


def description(request):
    with open(sys.argv[0], "rb") as source:
        executable_digest = "sha256:" + hashlib.sha256(source.read()).hexdigest()
    resources = [
        {"id": item["id"], "digest": item["digest"]}
        for item in request["configuration"]["resources"]
    ]
    adapter_fingerprint = fingerprint(
        {
            "format": "azimuth-adapter-fingerprint",
            "version": 1,
            "protocol_version": 1,
            "id": "synthetic",
            "provider_family": "synthetic/challenge-planning",
            "adapter_version": "0.1.0",
            "build": "challenge-planning-1",
            "content": {
                "executable_digest": executable_digest,
                "resources": resources,
            },
        }
    )
    capabilities = []
    for declared in CAPABILITIES:
        item = copy.deepcopy(declared)
        item["fingerprint"] = fingerprint(
            {
                "format": "azimuth-adapter-capability-fingerprint",
                "version": 1,
                "adapter_fingerprint": adapter_fingerprint,
                **declared,
            }
        )
        capabilities.append(item)
    result = {
        "format": "azimuth-adapter-description",
        "version": 1,
        "protocol_version": 1,
        "id": "synthetic",
        "provider_family": "synthetic/challenge-planning",
        "adapter_version": "0.1.0",
        "build": "challenge-planning-1",
        "content": {
            "executable_digest": executable_digest,
            "resources": resources,
        },
        "adapter_fingerprint": adapter_fingerprint,
        "capabilities": capabilities,
    }
    result["descriptor_fingerprint"] = fingerprint(
        {
            "format": "azimuth-adapter-descriptor-fingerprint",
            "version": 1,
            "descriptor": result,
        }
    )
    return result


def selection_fingerprint(selection):
    return fingerprint(
        {
            "format": "azimuth-run-selection-fingerprint",
            "version": 1,
            "plan_fingerprint": selection["plan_fingerprint"],
            "context": selection["context"],
            "checks": selection["checks"],
            "challenges": selection["challenges"],
        }
    )


def run_id(bundle):
    return fingerprint(
        {
            "format": "azimuth-run-identity",
            "version": 1,
            "source_system": bundle["provenance"]["source"]["system"],
            "source_execution": bundle["provenance"]["source"]["execution"],
            "subject_fingerprint": bundle["subject_fingerprint"],
            "plan_fingerprint": bundle["plan"]["fingerprint"],
            "launch_fingerprint": bundle["provenance"]["adapter"]["launch_fingerprint"],
        }
    )


def observation_fingerprint(bundle, execution):
    observation = execution["observation"]
    return fingerprint(
        {
            "format": "azimuth-observation-fingerprint",
            "version": 1,
            "run_id": bundle["run_id"],
            "subject_fingerprint": bundle["subject_fingerprint"],
            "check": execution["check"],
            "context": bundle["actual_selection"]["context"],
            "outcome": observation["outcome"],
            "observed_at_ms": observation["observed_at_ms"],
        }
    )


def challenge_fingerprint(bundle, execution):
    result = execution["result"]
    return fingerprint(
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


def bundle_fingerprint(bundle):
    payload = copy.deepcopy(bundle)
    payload.pop("bundle_fingerprint", None)
    return fingerprint(
        {
            "format": "azimuth-run-bundle-fingerprint",
            "version": 1,
            "bundle": payload,
        }
    )


def form_semantics(form, mode):
    semantics = {
        ("mutation", "survivor"): ("findings", "mutation/survived"),
        ("mutation", "zero"): ("inconclusive", "mutation/no-mutants"),
        ("broad-analysis", "broad-warning"): ("findings", "analyzer/warning"),
        ("broad-analysis", "broad-unsupported"): (
            "inconclusive",
            "analyzer/unsupported",
        ),
        ("fault-injection", "fault-injected"): ("findings", "fault/injected"),
        ("fault-injection", "fault-activation"): ("clean", "fault/activation"),
        ("fault-injection", "fault-inconclusive"): (
            "inconclusive",
            "fault/inconclusive",
        ),
    }
    return semantics.get((form, mode), ("clean", None))


def assert_routes(launch):
    expected_forms = {
        "mutation/search": ("mutation", "synthetic/challenge"),
        "broad/analyzer": ("broad-analysis", "synthetic/challenge"),
        "fault/injector": ("fault-injection", "synthetic/faults"),
    }
    projectable = {
        "check-implementation",
        "realization",
        "mechanism-implementation",
        "artifact",
        "surface-member",
        "enumeration",
    }
    routes = {
        item["selection"]["id"]: item
        for item in launch["routes"]
        if item["selection"]["kind"] == "challenge"
    }
    for challenge in launch["plan"]["challenges"]:
        route = routes[challenge["id"]]
        form, address = expected_forms[challenge["challenger"]["id"]]
        assert route["capability"]["challenge_form"] == form
        assert route["capability"]["address"] == address
        scoped = {
            (item["kind"], item["id"], item["fingerprint"])
            for item in challenge["scope"]["anchors"] + challenge["scope"]["inputs"]
            if item["kind"] in projectable
        }
        inputs = {
            (item["kind"], item["id"], item["fingerprint"])
            for item in route["inputs"]
        }
        assert inputs == scoped


def make_bundle(request):
    launch = request["launch_plan"]
    assert_routes(launch)
    planned = launch["planned_at_ms"]
    finished = planned + 2
    mode = os.environ["MODE"]
    omitted = mode == "omitted"
    checks = copy.deepcopy(launch["plan"]["checks"])
    planned_challenges = copy.deepcopy(launch["plan"]["challenges"])
    challenges = [
        item for item in planned_challenges
        if not omitted or item["lane"] == "gate"
    ]
    if mode == "drift" and challenges:
        challenges[0]["target"]["fingerprint"] = "sha256:" + "f" * 64
    actual = {
        "context": copy.deepcopy(launch["plan"]["required_context"]),
        "plan_fingerprint": launch["plan"]["fingerprint"],
        "checks": checks,
        "challenges": challenges,
    }
    actual["fingerprint"] = selection_fingerprint(actual)
    check_outcome = "inconclusive" if mode == "zero" else "satisfied"
    activities = []
    if checks or any(form_semantics(
        next(
            route["capability"]["challenge_form"]
            for route in launch["routes"]
            if route["selection"]["id"] == challenge["id"]
        ), mode
    )[0] != "inconclusive" for challenge in challenges):
        activities.append(
            {
                "id": "shared-analysis",
                "status": "failed" if mode == "zero" else "completed",
                "started_at_ms": planned + 1,
                "finished_at_ms": finished,
                "artifacts": [],
                "diagnostics": [],
                "attributes": {},
            }
        )
    bundle = {
        "format": "azimuth-run-bundle",
        "version": 1,
        "run_id": "sha256:" + "0" * 64,
        "bundle_revision": 0,
        "bundle_fingerprint": "sha256:" + "0" * 64,
        "subject": copy.deepcopy(launch["subject"]),
        "subject_fingerprint": launch["subject_fingerprint"],
        "planned_at_ms": planned,
        "started_at_ms": planned + 1,
        "finished_at_ms": finished,
        "status": "partial" if omitted else "complete",
        "plan": copy.deepcopy(launch["plan"]),
        "actual_selection": actual,
        "provenance": {
            "mode": launch["operation"],
            "source": {
                "system": "synthetic/challenge-planning",
                "execution": "short-lived-adapter",
            },
            "normalizer": {
                "id": "adapter/synthetic",
                "version": launch["adapter"]["adapter_version"],
                "build_fingerprint": launch["adapter"]["adapter_fingerprint"],
            },
            "adapter": {
                **copy.deepcopy(launch["adapter"]),
                "launch_fingerprint": launch["fingerprint"],
                "routes": copy.deepcopy(launch["routes"]),
                "import_inputs": [
                    {
                        "id": item["id"],
                        "digest": item["digest"],
                        "size_bytes": item["size_bytes"],
                    }
                    for item in request["inputs"]
                ],
            },
            "generated_at_ms": finished,
        },
        "artifacts": [],
        "diagnostics": [],
        "activities": activities,
        "check_executions": [],
        "challenger_executions": [],
    }
    if omitted:
        for challenge in planned_challenges:
            if challenge["lane"] != "scheduled":
                continue
            bundle["diagnostics"].append(
                {
                    "id": "deferred/" + challenge["id"].split("/", 1)[1],
                    "class": "execution",
                    "severity": "warning",
                    "code": "challenge/deferred",
                    "message": "The scheduled synthetic Challenge did not execute.",
                    "scope": {"kind": "challenge-selection", "id": challenge["id"]},
                    "artifacts": [],
                    "details": {},
                }
            )
    bundle["run_id"] = run_id(bundle)
    for check in checks:
        execution = {
            "check": {"id": check["id"], "fingerprint": check["fingerprint"]},
            "units": [
                {
                    "id": unit["id"],
                    "attempts": [
                        {
                            "ordinal": 1,
                            "activity": "shared-analysis",
                            "outcome": check_outcome,
                        }
                    ],
                }
                for unit in check["units"]
            ],
            "observation": {
                "outcome": check_outcome,
                "observed_at_ms": finished,
                "fingerprint": "sha256:" + "0" * 64,
                "artifacts": [],
                "diagnostics": [],
            },
        }
        execution["observation"]["fingerprint"] = observation_fingerprint(bundle, execution)
        bundle["check_executions"].append(execution)
    for index, challenge in enumerate(challenges):
        route = next(
            item for item in launch["routes"]
            if item["selection"]["id"] == challenge["id"]
        )
        form = route["capability"]["challenge_form"]
        outcome, code = form_semantics(form, mode)
        activity = "shared-analysis"
        if outcome == "inconclusive" and mode != "zero":
            activity = f"inconclusive-{index}"
            bundle["activities"].append(
                {
                    "id": activity,
                    "status": "failed",
                    "started_at_ms": planned + 1,
                    "finished_at_ms": finished,
                    "artifacts": [],
                    "diagnostics": [],
                    "attributes": {},
                }
            )
        objections = []
        result_diagnostics = []
        if code is not None:
            diagnostic = f"{code}-{index}"
            if outcome == "findings":
                objections.append(diagnostic)
            else:
                result_diagnostics.append(diagnostic)
            bundle["diagnostics"].append(
                {
                    "id": diagnostic,
                    "class": "objection" if outcome == "findings" else "execution",
                    "severity": "warning",
                    "code": code,
                    "message": f"Synthetic {form} reported {outcome}.",
                    "scope": {
                        "kind": "challenger-execution",
                        "challenger_fingerprint": challenge["challenger"]["fingerprint"],
                        "target_fingerprint": challenge["target"]["fingerprint"],
                    },
                    "artifacts": [],
                    "details": {},
                }
            )
        execution = {
            "challenge": challenge["id"],
            "challenger": copy.deepcopy(challenge["challenger"]),
            "target": copy.deepcopy(challenge["target"]),
            "units": [
                {
                    "id": unit["id"],
                    "attempts": [
                        {"ordinal": 1, "activity": activity, "outcome": outcome}
                    ],
                }
                for unit in challenge["units"]
            ],
            "result": {
                "outcome": outcome,
                "observed_at_ms": finished,
                "fingerprint": "sha256:" + "0" * 64,
                "objections": objections,
                "artifacts": [],
                "diagnostics": result_diagnostics,
            },
        }
        execution["result"]["fingerprint"] = challenge_fingerprint(bundle, execution)
        bundle["challenger_executions"].append(execution)
        if code is not None:
            next(item for item in bundle["activities"] if item["id"] == activity)[
                "diagnostics"
            ].append(diagnostic)
    bundle["activities"].sort(key=lambda item: item["id"])
    bundle["bundle_fingerprint"] = bundle_fingerprint(bundle)
    return bundle


if os.environ.get("SYNTHETIC_LITERAL") != "exact-value":
    raise SystemExit("configured literal was not delivered exactly")
if "HOME" in os.environ or "PATH" in os.environ:
    raise SystemExit("ambient environment reached the adapter")
if REQUEST["format"] != "azimuth-adapter-request" or REQUEST["version"] != 1:
    raise SystemExit("unexpected adapter request")
adapter_description = description(REQUEST)
response = {
    "format": "azimuth-adapter-response",
    "version": 1,
    "request_id": REQUEST["request_id"],
    "operation": REQUEST["operation"],
    "status": "ok",
    "description": adapter_description,
}
if REQUEST["operation"] != "describe":
    response["launch_fingerprint"] = REQUEST["launch_plan"]["fingerprint"]
    response["bundle"] = make_bundle(REQUEST)
print(json.dumps(response, ensure_ascii=False, separators=(",", ":"), sort_keys=True))
