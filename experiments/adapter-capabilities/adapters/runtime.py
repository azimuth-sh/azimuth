"""Standard-library implementation shared by the staged synthetic adapters."""

import copy
import hashlib
import json
import os
import sys
import time


def canonical(value):
    return json.dumps(value, ensure_ascii=False, separators=(",", ":"), sort_keys=True)


def fingerprint(value):
    return "sha256:" + hashlib.sha256(canonical(value).encode("utf-8")).hexdigest()


def definitions(role):
    if role == "executor":
        return {
            "id": "executor",
            "provider_family": "synthetic/execute",
            "capabilities": [
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
            ],
        }
    return {
        "id": "importer",
        "provider_family": "synthetic/import",
        "capabilities": [
            {
                "id": "reports",
                "classes": ["check.import"],
                "challenge_forms": [],
                "semantic_settings": {},
            }
        ],
    }


def description(role, request):
    definition = definitions(role)
    build = "drifted-build" if os.environ.get("MODE") == "descriptor-drift" else (
        "adapter-conformance-1"
    )
    with open(sys.argv[0], "rb") as source:
        executable_digest = "sha256:" + hashlib.sha256(source.read()).hexdigest()
    resources = [
        {"id": item["id"], "digest": item["digest"]}
        for item in request["configuration"]["resources"]
    ]
    adapter_preimage = {
        "format": "azimuth-adapter-fingerprint",
        "version": 1,
        "protocol_version": 1,
        "id": definition["id"],
        "provider_family": definition["provider_family"],
        "adapter_version": "0.2.0-alpha.2",
        "build": build,
        "content": {
            "executable_digest": executable_digest,
            "resources": resources,
        },
    }
    adapter_fingerprint = fingerprint(adapter_preimage)
    requested = {
        item["address"].split("/", 1)[1]: item
        for item in request["configuration"]["capabilities"]
    }
    capabilities = []
    for item in definition["capabilities"]:
        capability = copy.deepcopy(item)
        capability["fingerprint"] = fingerprint(
            {
                "format": "azimuth-adapter-capability-fingerprint",
                "version": 1,
                "adapter_fingerprint": adapter_fingerprint,
                **item,
            }
        )
        if item["id"] in requested and os.environ.get("MODE") != "descriptor-drift":
            assert capability["fingerprint"] == requested[item["id"]]["fingerprint"]
        capabilities.append(capability)
    result = {
        "format": "azimuth-adapter-description",
        "version": 1,
        "protocol_version": 1,
        "id": definition["id"],
        "provider_family": definition["provider_family"],
        "adapter_version": "0.2.0-alpha.2",
        "build": build,
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


def valid_fingerprint(value):
    if not isinstance(value, str) or not value.startswith("sha256:"):
        return False
    digest = value.removeprefix("sha256:")
    return len(digest) == 64 and all(character in "0123456789abcdef" for character in digest)


def validate_request(request):
    assert request["format"] == "azimuth-adapter-request"
    assert request["version"] == 1
    operation = request["operation"]
    if operation == "describe":
        adapter = request["adapter"]
        expected = fingerprint(
            {
                "format": "azimuth-adapter-request-fingerprint",
                "version": 1,
                "operation": "describe",
                "adapter": {
                    "id": adapter["id"],
                    "configuration_fingerprint": adapter["configuration_fingerprint"],
                },
            }
        )
        assert request["request_id"] == expected
        return

    assert operation in {"execute", "import"}
    launch = request["launch_plan"]
    assert launch["operation"] == operation
    inputs = request["inputs"]
    input_accounts = [
        {"id": item["id"], "digest": item["digest"], "size_bytes": item["size_bytes"]}
        for item in inputs
    ]
    assert [item["id"] for item in input_accounts] == sorted(
        item["id"] for item in input_accounts
    )
    assert (operation == "execute" and not inputs) or (operation == "import" and inputs)
    predecessors = request["predecessors"]
    for revision, predecessor in enumerate(predecessors):
        assert set(predecessor) == {"bundle_revision", "bundle_fingerprint"}
        assert predecessor["bundle_revision"] == revision
        assert valid_fingerprint(predecessor["bundle_fingerprint"])
    terminal = request["terminal_predecessor"]
    if not predecessors:
        assert terminal is None
    else:
        assert terminal is not None
        last = predecessors[-1]
        assert terminal["bundle_revision"] == last["bundle_revision"]
        assert terminal["bundle_fingerprint"] == last["bundle_fingerprint"]
        assert bundle_fingerprint(terminal) == last["bundle_fingerprint"]
    expected = fingerprint(
        {
            "format": "azimuth-adapter-request-fingerprint",
            "version": 1,
            "operation": operation,
            "launch_fingerprint": launch["fingerprint"],
            "inputs": input_accounts,
            "predecessors": predecessors,
        }
    )
    assert request["request_id"] == expected


def input_identities(role, request):
    identities = []
    imported_outcome = None
    for item in request["inputs"]:
        with open(item["locator"], "rb") as source:
            content = source.read()
        assert len(content) == item["size_bytes"]
        assert "sha256:" + hashlib.sha256(content).hexdigest() == item["digest"]
        identities.append(
            {"id": item["id"], "digest": item["digest"], "size_bytes": item["size_bytes"]}
        )
        if role == "importer":
            native = json.loads(content)
            imported_outcome = native["outcome"]
            assert imported_outcome in {"satisfied", "violated", "inconclusive"}
    return identities, imported_outcome


def make_bundle(role, request, adapter_description):
    launch = request["launch_plan"]
    terminal = request["terminal_predecessor"]
    identities, imported_outcome = input_identities(role, request)
    if terminal is not None:
        result = copy.deepcopy(terminal)
        result["bundle_revision"] += 1
        result["corrects"] = terminal["bundle_fingerprint"]
        result["correction_reason"] = "the synthetic source account was normalized again"
        result["provenance"]["adapter"]["import_inputs"] = identities
        result["bundle_fingerprint"] = bundle_fingerprint(result)
        return result

    mode = os.environ["MODE"]
    planned = launch["planned_at_ms"]
    started = planned + 1
    finished = planned + 2
    empty = mode in {"partial", "cancelled"}
    timed_out = mode == "timed-out-fact"
    actual = {
        "context": copy.deepcopy(launch["plan"]["required_context"]),
        "plan_fingerprint": launch["plan"]["fingerprint"],
        "checks": [] if empty else copy.deepcopy(launch["plan"]["checks"]),
        "challenges": [] if empty else copy.deepcopy(launch["plan"]["challenges"]),
    }
    actual["fingerprint"] = selection_fingerprint(actual)
    status = mode if mode in {"partial", "cancelled"} else "complete"
    if timed_out:
        status = "timed-out"
    activity_status = "timed-out" if timed_out else "completed"
    activities = [] if empty else [
        {
            "id": "shared-work",
            "status": activity_status,
            "started_at_ms": started,
            "finished_at_ms": finished,
            "artifacts": [],
            "diagnostics": [],
            "attributes": {},
        }
    ]
    bundle = {
        "format": "azimuth-run-bundle",
        "version": 1,
        "run_id": "sha256:" + "0" * 64,
        "bundle_revision": 0,
        "bundle_fingerprint": "sha256:" + "0" * 64,
        "subject": copy.deepcopy(launch["subject"]),
        "subject_fingerprint": launch["subject_fingerprint"],
        "planned_at_ms": planned,
        "started_at_ms": started,
        "finished_at_ms": finished,
        "status": status,
        "plan": copy.deepcopy(launch["plan"]),
        "actual_selection": actual,
        "provenance": {
            "mode": launch["operation"],
            "source": {
                "system": "synthetic-adapter",
                "execution": "adapter-capability-conformance",
            },
            "normalizer": {
                "id": "adapter/" + launch["adapter"]["id"],
                "version": launch["adapter"]["adapter_version"],
                "build_fingerprint": launch["adapter"]["adapter_fingerprint"],
            },
            "adapter": {
                **copy.deepcopy(launch["adapter"]),
                "launch_fingerprint": launch["fingerprint"],
                "routes": copy.deepcopy(launch["routes"]),
                "import_inputs": identities,
            },
            "generated_at_ms": finished,
        },
        "artifacts": [],
        "diagnostics": [],
        "activities": activities,
        "check_executions": [],
        "challenger_executions": [],
    }
    bundle["run_id"] = run_id(bundle)
    check_outcome = "inconclusive" if timed_out else "satisfied"
    if imported_outcome is not None:
        check_outcome = imported_outcome
    if mode == "violated":
        check_outcome = "violated"
    if not empty:
        for check in actual["checks"]:
            execution = {
                "check": {"id": check["id"], "fingerprint": check["fingerprint"]},
                "units": [
                    {
                        "id": unit["id"],
                        "attempts": [
                            {
                                "ordinal": 1,
                                "activity": "shared-work",
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
        for challenge in actual["challenges"]:
            challenge_outcome = "inconclusive" if timed_out else "findings"
            objections = []
            if challenge_outcome == "findings":
                objection_id = "challenge/synthetic-finding"
                objections = [objection_id]
                bundle["diagnostics"].append(
                    {
                        "id": objection_id,
                        "class": "objection",
                        "severity": "warning",
                        "code": "fault-injection/detected",
                        "message": "The synthetic challenge found a controlled objection.",
                        "scope": {
                            "kind": "challenger-execution",
                            "challenger_fingerprint": challenge["challenger"]["fingerprint"],
                            "target_fingerprint": challenge["target"]["fingerprint"],
                        },
                        "artifacts": [],
                        "details": {"source": "synthetic"},
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
                            {
                                "ordinal": 1,
                                "activity": "shared-work",
                                "outcome": challenge_outcome,
                            }
                        ],
                    }
                    for unit in challenge["units"]
                ],
                "result": {
                    "outcome": challenge_outcome,
                    "observed_at_ms": finished,
                    "fingerprint": "sha256:" + "0" * 64,
                    "objections": objections,
                    "artifacts": [],
                    "diagnostics": [],
                },
            }
            execution["result"]["fingerprint"] = challenge_fingerprint(bundle, execution)
            bundle["challenger_executions"].append(execution)
    bundle["bundle_fingerprint"] = bundle_fingerprint(bundle)
    return bundle


mode = os.environ.get("MODE", "normal")
if os.environ.get("SYNTHETIC_LITERAL") != "exact-value":
    raise SystemExit("configured literal was not delivered exactly")
if "HOME" in os.environ or "PATH" in os.environ:
    raise SystemExit("ambient environment reached the adapter")
validate_request(REQUEST)
if mode == "nonzero":
    print("synthetic nonzero exit", file=sys.stderr)
    raise SystemExit(7)
if mode == "hang":
    time.sleep(2)
if mode == "stdout-overflow":
    print("x" * 20000)
    raise SystemExit(0)
if mode == "stderr-overflow":
    print("x" * 20000, file=sys.stderr)
    time.sleep(1)
    raise SystemExit(0)
if mode == "malformed":
    print("{")
    raise SystemExit(0)
if mode == "schema":
    print('{"format":"azimuth-adapter-response"}')
    raise SystemExit(0)

adapter_description = description(ROLE, REQUEST)
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
    response["bundle"] = make_bundle(ROLE, REQUEST, adapter_description)
print(json.dumps(response, ensure_ascii=False, separators=(",", ":"), sort_keys=True))
if mode == "extra-output":
    print("{}")
