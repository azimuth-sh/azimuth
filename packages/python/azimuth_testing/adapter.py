#!/usr/bin/python3
"""Pinned import adapter; staged resources are verified before interpretation."""
import hashlib
import json
from pathlib import Path
import sys
import types


def strict_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("duplicate JSON field")
        result[key] = value
    return result


def read_json(content):
    return json.loads(content, object_pairs_hook=strict_object,
                      parse_constant=lambda value: (_ for _ in ()).throw(ValueError("invalid JSON constant")))


def fields(value, expected):
    if not isinstance(value, dict) or set(value) != set(expected):
        raise ValueError("unexpected adapter protocol fields")


def digest(content):
    return "sha256:" + hashlib.sha256(content).hexdigest()


def description(resources, helper):
    identity = dict(protocol_version=1, id="azimuth-tests", provider_family="azimuth/testing",
                    adapter_version="0.1.0-alpha.1", build="typed-test-import",
                    content={"executable_digest": digest(Path(__file__).read_bytes()),
                             "resources": [{"id": resource["id"], "digest": resource["digest"]} for resource in resources]})
    adapter = helper.fingerprint(dict(format="azimuth-adapter-fingerprint", version=1, **identity))
    capability = {"id": "reports", "classes": ["check.import"], "challenge_forms": [], "semantic_settings": {}}
    capability["fingerprint"] = helper.fingerprint(dict(format="azimuth-adapter-capability-fingerprint", version=1,
                                                       adapter_fingerprint=adapter, **capability))
    result = dict(format="azimuth-adapter-description", version=1, **identity,
                  adapter_fingerprint=adapter, capabilities=[capability])
    result["descriptor_fingerprint"] = helper.fingerprint({"format": "azimuth-adapter-descriptor-fingerprint", "version": 1, "descriptor": result})
    return result


def main():
    request = read_json(sys.stdin.buffer.read())
    operation = request["operation"]
    fields(request, ("format", "version", "request_id", "operation", "adapter", "configuration") if operation == "describe"
           else ("format", "version", "request_id", "operation", "launch_plan", "configuration", "inputs", "predecessors", "terminal_predecessor"))
    if request["format"] != "azimuth-adapter-request" or request["version"] != 1 or operation not in ("describe", "import"):
        raise ValueError("unsupported adapter request")
    configuration = request["configuration"]
    fields(configuration, ("fingerprint", "semantic_settings", "resources", "capabilities"))
    if configuration["semantic_settings"] != {"native_contract": "azimuth-test-report-v1"}:
        raise ValueError("unsupported configured test contract")
    resources = configuration["resources"]
    if [resource["id"] for resource in resources] != ["run-protocol", "test-normalizer"]:
        raise ValueError("adapter requires pinned run-protocol and test-normalizer resources")
    modules = []
    for resource, name in zip(resources, ("azimuth_run", "azimuth_testing_normalizer")):
        fields(resource, ("id", "digest", "locator"))
        content = Path(resource["locator"]).read_bytes()
        if digest(content) != resource["digest"]:
            raise ValueError("pinned adapter resource digest mismatch")
        module = types.ModuleType(name)
        sys.modules[name] = module
        exec(compile(content, resource["locator"], "exec"), module.__dict__)
        modules.append(module)
    helper, normalizer = modules
    descriptor = description(resources, helper)
    expected_configuration = helper.fingerprint(dict(format="azimuth-adapter-configuration-fingerprint", version=1,
        adapter_fingerprint=descriptor["adapter_fingerprint"], descriptor_fingerprint=descriptor["descriptor_fingerprint"],
        semantic_settings=configuration["semantic_settings"], environment={"literals": {}},
        limits={"timeout_ms": 30000, "stdout_bytes": 16777216, "stderr_bytes": 1048576}, capabilities=descriptor["capabilities"]))
    if configuration["fingerprint"] != expected_configuration:
        raise ValueError("adapter configuration fingerprint mismatch")
    if configuration["capabilities"] != [{"address": "azimuth-tests/reports", "fingerprint": descriptor["capabilities"][0]["fingerprint"], "semantic_settings": {}}]:
        raise ValueError("adapter capability mismatch")
    response = dict(format="azimuth-adapter-response", version=1, request_id=request["request_id"],
                    operation=operation, status="ok", description=descriptor)
    if operation == "describe":
        if request["adapter"] != {"id": "azimuth-tests", "configuration_fingerprint": configuration["fingerprint"]}:
            raise ValueError("adapter identity mismatch")
        expected_request = helper.fingerprint(dict(format="azimuth-adapter-request-fingerprint", version=1, operation="describe", adapter=request["adapter"]))
    else:
        launch = request["launch_plan"]
        if len(launch["adapters"]) != 1:
            raise ValueError("provider requires one partitioned launch")
        selected = launch["adapters"][0]
        if selected["configuration_fingerprint"] != configuration["fingerprint"] or any(selected[key] != descriptor[key] for key in ("id", "adapter_version", "adapter_fingerprint", "descriptor_fingerprint")):
            raise ValueError("adapter launch substitution")
        inputs = request["inputs"]
        input_ids = [item["id"] for item in inputs]
        if len(input_ids) != len(set(input_ids)) or not {"mapping", "native-results", "report"} <= set(input_ids) or set(input_ids) - {"mapping", "native-results", "report", "selection", "correction"}:
            raise ValueError("import requires report, mapping, native-results and optional selection/correction")
        documents = {}
        identities = []
        for item in inputs:
            fields(item, ("id", "digest", "size_bytes", "locator"))
            content = Path(item["locator"]).read_bytes()
            if digest(content) != item["digest"] or len(content) != item["size_bytes"]:
                raise ValueError("import input content mismatch")
            documents[item["id"]] = normalizer.load_json(content)
            identities.append({key: item[key] for key in ("id", "digest", "size_bytes")})
        native = documents["native-results"]
        fields(native, ("format", "version", "framework", "documents"))
        if native["format"] != "azimuth-native-test-results" or native["version"] != 1 or native["framework"] != documents["report"]["framework"]:
            raise ValueError("native report framework substitution")
        native_results = normalizer.parse_native_documents(native)
        normalized = normalizer.normalize_report(launch, documents["report"], documents["mapping"], native_results=native_results,
                         import_inputs=identities, generated_at_ms=max([documents["report"]["finished_at_ms"]] + [item["finished_at_ms"] for item in native_results]), selection=documents.get("selection"))
        bundle = normalized["bundle"]
        predecessors = request["predecessors"]
        terminal = request["terminal_predecessor"]
        if predecessors:
            correction = documents.get("correction")
            fields(correction, ("reason",))
            reason = normalizer.text(correction["reason"])
            if not terminal or predecessors[-1] != {key: terminal[key] for key in ("bundle_revision", "bundle_fingerprint")}:
                raise ValueError("correction predecessor identity mismatch")
            for revision, previous in enumerate(predecessors):
                fields(previous, ("bundle_revision", "bundle_fingerprint"))
                if previous["bundle_revision"] != revision:
                    raise ValueError("correction history is not contiguous")
            terminal_payload = {key: value for key, value in terminal.items() if key != "bundle_fingerprint"}
            if helper.fingerprint(dict(format="azimuth-run-bundle-fingerprint", version=2, bundle=terminal_payload)) != terminal["bundle_fingerprint"]:
                raise ValueError("correction predecessor content mismatch")
            for key in ("run_id", "verification_target", "verification_target_fingerprint", "plan", "planned_at_ms", "started_at_ms"):
                if terminal[key] != bundle[key]:
                    raise ValueError("correction changes an execution anchor")
            if terminal["actual_selection"]["execution_context"] != bundle["actual_selection"]["execution_context"] or terminal["provenance"]["normalizer"] != bundle["provenance"]["normalizer"]:
                raise ValueError("correction changes Execution Context or normalizer")
            if terminal["provenance"]["source"] != bundle["provenance"]["source"]:
                raise ValueError("correction changes native execution identity")
            for key, value in terminal["provenance"]["adapter"].items():
                if key != "import_inputs" and bundle["provenance"]["adapter"].get(key) != value:
                    raise ValueError("correction changes adapter anchors")
            bundle.update(bundle_revision=len(predecessors), corrects=terminal["bundle_fingerprint"],
                          correction_reason=reason)
            del bundle["bundle_fingerprint"]
            bundle["bundle_fingerprint"] = helper.fingerprint(dict(format="azimuth-run-bundle-fingerprint", version=2, bundle=bundle))
        elif terminal is not None or "correction" in documents:
            raise ValueError("initial import cannot supply correction predecessor or reason")
        expected_request = helper.fingerprint(dict(format="azimuth-adapter-request-fingerprint", version=1, operation="import",
             launch_fingerprint=launch["fingerprint"], inputs=identities, predecessors=predecessors))
        response.update(launch_fingerprint=launch["fingerprint"], bundle=bundle)
    if request["request_id"] != expected_request:
        raise ValueError("adapter request fingerprint mismatch")
    sys.stdout.write(helper.canonical_json(response) + "\n")


if __name__ == "__main__":
    try:
        main()
    except (ValueError, KeyError, TypeError, OSError, json.JSONDecodeError):
        print("typed test import rejected; no valid Run emitted", file=sys.stderr)
        sys.exit(1)
