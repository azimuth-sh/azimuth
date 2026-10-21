"""Explicit native test mappings preserve inconclusive and adverse execution facts."""
from copy import deepcopy
from datetime import datetime
import hashlib
import json
import re
import xml.etree.ElementTree as ET
from azimuth_run import build_check_bundle, canonical_json, fingerprint

OUTCOMES = {"passed", "failed", "skipped", "timed-out", "interrupted"}
MAX_INTEGER = 9007199254740991


def load_json(content):
    def pairs(items):
        result = {}
        for key, value in items:
            if key in result:
                raise ValueError("duplicate JSON field: " + key)
            result[key] = value
        return result
    return json.loads(content, object_pairs_hook=pairs,
                      parse_constant=lambda value: (_ for _ in ()).throw(ValueError("invalid JSON constant")))


def fields(value, expected):
    if not isinstance(value, dict) or set(value) != set(expected):
        raise ValueError("unexpected object fields; expected " + ", ".join(sorted(expected)))


def text(value):
    if not isinstance(value, str) or not value.strip():
        raise ValueError("expected nonempty string")
    return value


def integer(value):
    if type(value) is not int or not 0 <= value <= MAX_INTEGER:
        raise ValueError("expected nonnegative safe integer")
    return value


def digest(value):
    if not isinstance(value, str) or not re.fullmatch(r"sha256:[0-9a-f]{64}", value):
        raise ValueError("expected canonical SHA-256 digest")
    return value


def array(value):
    if not isinstance(value, list):
        raise ValueError("expected array")
    return value


def timestamp(value):
    normalized = re.sub(r"(\.\d{6})\d+(?=Z|[+-]\d{2}:\d{2}$)", r"\1", value)
    result = datetime.fromisoformat(normalized.replace("Z", "+00:00"))
    if result.tzinfo is None:
        raise ValueError("native execution timestamp requires timezone")
    return integer(int(result.timestamp() * 1000))


def validate_context(report):
    fields(report, ("format", "version", "contexts"))
    if report["format"] != "azimuth-execution-context" or type(report["version"]) is not int or report["version"] != 2:
        raise ValueError("unsupported Execution Context format")
    if not isinstance(report["contexts"], dict):
        raise ValueError("contexts requires object")
    for name, context in report["contexts"].items():
        text(name)
        fields(context, ("contract_revision", "values"))
        digest(context["contract_revision"])
        if not isinstance(context["values"], dict):
            raise ValueError("context values requires object")
        for key, value in context["values"].items():
            text(key)
            if not isinstance(value, (str, bool)):
                integer(value)


def validate_report(report):
    fields(report, ("format", "version", "framework", "execution", "started_at_ms", "finished_at_ms", "tests"))
    if report["format"] != "azimuth-test-report" or type(report["version"]) is not int or report["version"] != 1:
        raise ValueError("unsupported test report")
    if report["framework"] not in ("xunit-v2", "playwright"):
        raise ValueError("unsupported test framework")
    text(report["execution"])
    start, finish = integer(report["started_at_ms"]), integer(report["finished_at_ms"])
    if finish < start:
        raise ValueError("report interval is reversed")
    seen = set()
    for test in array(report["tests"]):
        fields(test, ("id", "attempt", "outcome", "started_at_ms", "finished_at_ms", "contexts", "diagnostics"))
        identity = text(test["id"]), integer(test["attempt"])
        if identity in seen:
            raise ValueError("duplicate test identity / attempt")
        seen.add(identity)
        if test["outcome"] not in OUTCOMES:
            raise ValueError("unknown test outcome")
        if not start <= integer(test["started_at_ms"]) <= integer(test["finished_at_ms"]) <= finish:
            raise ValueError("test interval is outside report")
        for message in array(test["diagnostics"]):
            text(message)
        names = set()
        for context in array(test["contexts"]):
            fields(context, ("name", "before", "after"))
            name = text(context["name"])
            if name in names:
                raise ValueError("duplicate test context")
            names.add(name)
            for key in ("before", "after"):
                snapshot = context[key]
                fields(snapshot, ("captured_at_ms", "instance", "report"))
                integer(snapshot["captured_at_ms"])
                text(snapshot["instance"])
                validate_context(snapshot["report"])
                if set(snapshot["report"]["contexts"]) != {name}:
                    raise ValueError("capture must contain exactly its selected context")
            if not test["started_at_ms"] <= context["before"]["captured_at_ms"] <= context["after"]["captured_at_ms"] <= test["finished_at_ms"]:
                raise ValueError("capture interval lies outside test execution")
    for identity in {test["id"] for test in report["tests"]}:
        attempts = sorted(test["attempt"] for test in report["tests"] if test["id"] == identity)
        if attempts != list(range(len(attempts))):
            raise ValueError("test attempts require contiguous zero-based ordinals")


def validate_mapping(mapping):
    fields(mapping, ("format", "version", "framework", "source_system", "tests"))
    if mapping["format"] != "azimuth-test-mapping" or type(mapping["version"]) is not int or mapping["version"] != 2:
        raise ValueError("unsupported mapping")
    if mapping["framework"] not in ("xunit-v2", "playwright"):
        raise ValueError("unsupported mapping framework")
    text(mapping["source_system"])
    seen = set()
    native = set()
    for item in array(mapping["tests"]):
        if not isinstance(item, dict) or set(item) not in ({"id", "native_id", "check", "unit", "scenarios", "components", "contexts"}, {"id", "native_id", "check", "unit", "scenarios", "components", "contexts", "failed_outcome"}):
            raise ValueError("unexpected test mapping fields")
        for key in ("id", "native_id", "check", "unit"):
            text(item[key])
        identity = item["id"], item["check"], item["unit"]
        if identity in seen:
            raise ValueError("duplicate test mapping")
        seen.add(identity)
        native.add(item["native_id"])
        for key in ("scenarios", "components", "contexts"):
            values = array(item[key])
            if len(values) != len(set(values)):
                raise ValueError("duplicate mapping value")
            for value in values:
                digest(value) if key == "contexts" else text(value)
        if not item["scenarios"] or not item["components"]:
            raise ValueError("mapping requires explicit Scenario and component scope")
        if item.get("failed_outcome", "inconclusive") not in ("inconclusive", "violated"):
            raise ValueError("failed outcome requires an explicit inconclusive or reviewed violated interpretation")
    for identity in {item["id"] for item in mapping["tests"]}:
        if len({item["native_id"] for item in mapping["tests"] if item["id"] == identity}) != 1:
            raise ValueError("typed test identity maps to multiple native identities")
    if len(native) != len({item["id"] for item in mapping["tests"]}):
        raise ValueError("native identity maps to multiple typed test identities")


def parse_trx(content):
    if b"<!DOCTYPE" in (content if isinstance(content, bytes) else content.encode()):
        raise ValueError("TRX document type declarations are unsupported")
    root = ET.fromstring(content)
    local = lambda node: node.tag.rsplit("}", 1)[-1]
    definitions = {}
    for definition in root.iter():
        if local(definition) != "UnitTest":
            continue
        method = next((child for child in definition.iter() if local(child) == "TestMethod"), None)
        if method is None:
            raise ValueError("TRX definition has no TestMethod")
        key = definition.attrib["id"]
        if key in definitions:
            raise ValueError("duplicate TRX test definition")
        definitions[key] = text(method.attrib["className"]) + "." + text(method.attrib["name"])
    results = []
    seen = set()
    outcomes = {"Passed": "passed", "Failed": "failed", "NotExecuted": "skipped", "Timeout": "timed-out", "Aborted": "interrupted", "Error": "interrupted"}
    for result in root.iter():
        if local(result) != "UnitTestResult":
            continue
        execution = text(result.attrib["executionId"])
        if execution in seen:
            raise ValueError("duplicate TRX execution identity")
        seen.add(execution)
        method = definitions.get(result.attrib["testId"])
        if method is None:
            raise ValueError("TRX result has no native definition")
        native = method + "|" + text(result.attrib["testName"])
        outcome = outcomes.get(result.attrib.get("outcome"))
        if outcome is None:
            raise ValueError("unsupported TRX outcome")
        results.append({"native_id": native, "attempt": 0, "outcome": outcome,
                        "started_at_ms": timestamp(result.attrib["startTime"]),
                        "finished_at_ms": timestamp(result.attrib["endTime"])})
    return results


def parse_playwright(report):
    results = []
    def suite(node):
        for spec in node.get("specs", []):
            for test in spec.get("tests", []):
                if not isinstance(test["projectId"], str):
                    raise ValueError("Playwright project identity must be a string")
                native = text(spec["id"]) + "|" + test["projectId"]
                for result in test.get("results", []):
                    start = timestamp(result["startTime"])
                    duration = result["duration"]
                    if not isinstance(duration, (float, int)) or isinstance(duration, bool) or duration < 0:
                        raise ValueError("invalid Playwright duration")
                    status = result["status"]
                    outcome = "timed-out" if status == "timedOut" else status
                    if outcome not in OUTCOMES:
                        raise ValueError("unknown Playwright outcome")
                    results.append({"native_id": native, "attempt": integer(result["retry"]), "outcome": outcome,
                                    "started_at_ms": start, "finished_at_ms": start + __import__("math").ceil(duration)})
        for child in node.get("suites", []):
            suite(child)
    if not isinstance(report, dict) or "suites" not in report:
        raise ValueError("expected Playwright JSON reporter output")
    suite(report)
    return results


def _facts(report, mapping, native_results):
    validate_report(report)
    validate_mapping(mapping)
    if report["framework"] != mapping["framework"]:
        raise ValueError("mapping framework differs from report")
    native = {}
    for result in native_results:
        fields(result, ("native_id", "attempt", "outcome", "started_at_ms", "finished_at_ms"))
        key = text(result["native_id"]), integer(result["attempt"])
        if key in native or result["outcome"] not in OUTCOMES:
            raise ValueError("duplicate or invalid native test execution")
        if integer(result["finished_at_ms"]) < integer(result["started_at_ms"]):
            raise ValueError("native test interval is reversed")
        native[key] = result
    indexed = {(test["id"], test["attempt"]): deepcopy(test) for test in report["tests"]}
    selected = {item["id"]: item["native_id"] for item in mapping["tests"]}
    facts = {}
    for identity, native_id in selected.items():
        native_attempts = sorted((attempt, result) for (key, attempt), result in native.items() if key == native_id)
        typed_attempts = {attempt for (key, attempt) in indexed if key == identity}
        if typed_attempts - {attempt for attempt, _ in native_attempts}:
            raise ValueError("typed execution has no matching native runner result")
        attempts = []
        for attempt, result in native_attempts:
            fact = indexed.get((identity, attempt))
            if fact is None:
                fact = {"id": identity, "attempt": attempt, "outcome": result["outcome"],
                        "started_at_ms": result["started_at_ms"], "finished_at_ms": result["finished_at_ms"],
                        "contexts": [], "diagnostics": ["native-test-not-captured"]}
            elif fact["outcome"] != result["outcome"]:
                raise ValueError("typed report outcome differs from native runner")
            # TRX start times may be reconstructed from delayed result delivery, not body entry.
            elif fact["finished_at_ms"] < result["started_at_ms"] - 5000 or fact["started_at_ms"] > result["finished_at_ms"] + 5000:
                raise ValueError("typed execution interval is unrelated to native runner execution")
            attempts.append(fact)
        if [fact["attempt"] for fact in attempts] != list(range(len(attempts))):
            raise ValueError("native retries require contiguous zero-based ordinals")
        facts[identity] = attempts
    return facts


def _context_key(test):
    values = {}
    instances = {}
    for context in test["contexts"]:
        before, after = context["before"], context["after"]
        name = context["name"]
        if before["instance"] != after["instance"] or canonical_json(before["report"]) != canonical_json(after["report"]):
            raise ValueError("effective context or process lifetime changed during test")
        value = before["report"]["contexts"][name]
        key = "context/" + value["contract_revision"]
        if key in values:
            raise ValueError("duplicate context contract fingerprint")
        values[key] = fingerprint(value)
        instances[name] = before["instance"]
    return values, instances


def activity_identity(check, unit, ordinal):
    return "test-" + hashlib.sha256((check + "|" + unit + "|" + str(ordinal)).encode()).hexdigest()


def activity_context(values, activity):
    return {key + "/activity/" + activity: value for key, value in values.items()}


def partition_report(report, mapping, *, native_results):
    """Return explicit unit partitions; callers must plan each partition separately.

    These optional partitions preserve exact attempt selection. A full Run may also
    contain multiple activity-bound test units, each with independently pinned context.
    """
    facts = _facts(report, mapping, native_results)
    partitions = []
    plain = []
    for item in mapping["tests"]:
        attempts = facts[item["id"]]
        if item["contexts"]:
            for fact in attempts:
                values, instances = _context_key(fact)
                partitions.append({"tests": [{"id": item["id"], "attempt": fact["attempt"]}],
                                   "checks": {item["check"]: [item["unit"]]}, "context": activity_context(values, activity_identity(item["check"], item["unit"], 0)), "instances": instances})
        else:
            plain.append({"id": item["id"], "attempts": [fact["attempt"] for fact in attempts]})
    if plain:
        partitions.append({"tests": plain, "checks": {check: sorted({item["unit"] for item in mapping["tests"] if item["check"] == check and not item["contexts"]})
                                                      for check in sorted({item["check"] for item in mapping["tests"] if not item["contexts"]})}, "context": {}, "instances": {}})
    return partitions


def normalize_report(launch, report, mapping, *, native_results, import_inputs, generated_at_ms, predecessors=(), correction_reason=None, selection=None):
    facts = _facts(report, mapping, native_results)
    selected = {(check["id"], unit["id"]): check for check in launch["plan"]["checks"] for unit in check["units"]}
    selected_mapping = [item for item in mapping["tests"] if (item["check"], item["unit"]) in selected]
    if {(item["check"], item["unit"]) for item in selected_mapping} != set(selected):
        raise ValueError("every selected unit requires explicit native test mapping")
    if launch["plan"]["challenges"]:
        raise ValueError("test adapter accepts Check-only launches")
    execution = report["execution"]
    if selection is not None:
        choices = {}
        for choice in array(selection):
            fields(choice, ("id", "attempt"))
            identity, attempt = text(choice["id"]), integer(choice["attempt"])
            if identity in choices or identity not in facts:
                raise ValueError("partition selection contains duplicate or unknown test identity")
            choices[identity] = attempt
        if set(choices) != {item["id"] for item in selected_mapping}:
            raise ValueError("partition must cover exactly the tests mapped to selected units")
        for identity, attempt in choices.items():
            facts[identity] = [fact for fact in facts[identity] if fact["attempt"] == attempt]
            if not facts[identity]:
                raise ValueError("partition names a missing native test attempt")
        execution += "/partition/" + fingerprint(selection)[7:]
    all_facts = [fact for item in selected_mapping for fact in facts[item["id"]]]
    for item in selected_mapping:
        if not set(item["scenarios"]) <= set(selected[item["check"], item["unit"]]["scenarios"]):
            raise ValueError("mapping names an unselected Scenario")
    contexts = {}
    captures = []
    for item in selected_mapping:
        if item["contexts"] and sum(other["check"] == item["check"] and other["unit"] == item["unit"] for other in selected_mapping) != 1:
            raise ValueError("context-bearing units require one explicitly mapped native test")
        for ordinal, fact in enumerate(facts[item["id"]]):
            activity = activity_identity(item["check"], item["unit"], ordinal)
            try:
                values, _ = _context_key(fact)
            except ValueError:
                values = {}
                fact["diagnostics"].append("effective-context-changed")
            required = {"context/" + fp for fp in item["contexts"]}
            if set(values) != required:
                fact["diagnostics"].append("required-effective-context-not-captured")
            contexts.update(activity_context(values, activity))
            for context in fact["contexts"] if not fact["diagnostics"] else []:
                for component in item["components"]:
                    captures.append((component, context, activity))
    actual_context = dict(launch["plan"]["required_execution_context"])
    for key, value in contexts.items():
        if key in actual_context and actual_context[key] != value:
            raise ValueError("captured context differs from planned context")
        actual_context[key] = value
    if actual_context != launch["plan"]["required_execution_context"]:
        raise ValueError("context fingerprints must be explicitly pinned in the planned launch")
    start = min((fact["started_at_ms"] for fact in all_facts), default=report["started_at_ms"])
    finish = max((fact["finished_at_ms"] for fact in all_facts), default=report["finished_at_ms"])
    if not launch["planned_at_ms"] <= start <= finish <= integer(generated_at_ms):
        raise ValueError("execution timestamps do not fit the exact launch")
    activities, diagnostics, units = [], [], {}
    for check in launch["plan"]["checks"]:
        units[check["id"]] = []
        for unit in check["units"]:
            items = [item for item in selected_mapping if item["check"] == check["id"] and item["unit"] == unit["id"]]
            count = max((len(facts[item["id"]]) for item in items), default=0)
            attempts = []
            for ordinal in range(count):
                activity_id = activity_identity(check["id"], unit["id"], ordinal)
                outcomes = {}
                messages = []
                for scenario in check["scenarios"]:
                    results = []
                    for item in items:
                        if scenario not in item["scenarios"]:
                            continue
                        attempts_for_test = facts[item["id"]]
                        if not attempts_for_test:
                            results.append("inconclusive")
                            messages.append("selected-native-test-missing:" + item["id"])
                            continue
                        fact = attempts_for_test[min(ordinal, len(attempts_for_test) - 1)]
                        if fact["diagnostics"]:
                            result = "inconclusive"
                            messages.extend(fact["diagnostics"])
                        elif fact["outcome"] == "passed":
                            result = "satisfied"
                        elif fact["outcome"] == "failed":
                            result = item.get("failed_outcome", "inconclusive")
                        else:
                            result = "inconclusive"
                            messages.append("native-test-" + fact["outcome"] + ":" + item["id"])
                        results.append(result)
                    outcomes[scenario] = "violated" if "violated" in results else "satisfied" if results and set(results) == {"satisfied"} else "inconclusive"
                diagnostic_ids = []
                for index, message in enumerate(sorted(set(messages))):
                    diagnostic_id = activity_id + "-" + str(index)
                    diagnostics.append({"id": diagnostic_id, "class": "normalization", "severity": "warning",
                                        "code": "test-evidence-inconclusive", "message": message,
                                        "scope": {"kind": "check-execution", "check": check["id"]}, "artifacts": [], "details": {}})
                    diagnostic_ids.append(diagnostic_id)
                round_facts = [facts[item["id"]][min(ordinal, len(facts[item["id"]]) - 1)] for item in items if facts[item["id"]]]
                activity_start = min((fact["started_at_ms"] for fact in round_facts), default=start)
                activity_finish = max((fact["finished_at_ms"] for fact in round_facts), default=finish)
                activity_captures = [context for _, context, identity in captures if identity == activity_id]
                if activity_captures:
                    activity_start = max(context["before"]["captured_at_ms"] for context in activity_captures)
                    activity_finish = min(context["after"]["captured_at_ms"] for context in activity_captures)
                    if activity_start > activity_finish:
                        raise ValueError("activity context captures do not share an execution interval")
                activities.append({"id": activity_id, "status": "completed", "started_at_ms": activity_start, "finished_at_ms": activity_finish,
                                   "artifacts": [], "diagnostics": diagnostic_ids, "attributes": {"native-round": str(ordinal)}})
                attempts.append({"ordinal": ordinal, "activity": activity_id, "outcomes": outcomes})
            units[check["id"]].append({"id": unit["id"], "attempts": attempts})
    adapter = launch["adapters"][0]
    bundle = build_check_bundle(launch, source={"system": mapping["source_system"], "execution": execution},
                                normalizer={"id": "adapter/" + adapter["id"], "version": adapter["adapter_version"], "build_fingerprint": adapter["adapter_fingerprint"]},
                                started_at_ms=start, finished_at_ms=finish, generated_at_ms=generated_at_ms,
                                actual_context=actual_context, activities=activities, units_by_check=units, artifacts=[], diagnostics=diagnostics,
                                import_inputs=import_inputs, predecessors=predecessors, correction_reason=correction_reason)
    sidecar = []
    for component, context, activity in captures:
        capture = {"kind": "execution", "activity": activity, "run": bundle["run_id"], "bundle_fingerprint": bundle["bundle_fingerprint"],
                   "verification_target_fingerprint": bundle["verification_target_fingerprint"], "component": component,
                   "instance": context["before"]["instance"], "started_at_ms": context["before"]["captured_at_ms"],
                   "finished_at_ms": context["after"]["captured_at_ms"], "before": context["before"], "after": context["after"]}
        if capture not in sidecar:
            sidecar.append(capture)
    return {"bundle": bundle, "context_captures": sidecar}


def parse_native_documents(native):
    fields(native, ("format", "version", "framework", "documents"))
    if native["format"] != "azimuth-native-test-results" or type(native["version"]) is not int or native["version"] != 1:
        raise ValueError("unsupported native test document envelope")
    results = []
    for document in array(native["documents"]):
        fields(document, ("format", "content"))
        if document["format"] == "trx" and native["framework"] == "xunit-v2":
            parsed = parse_trx(document["content"])
            offsets = {}
            for result in results:
                offsets[result["native_id"]] = max(offsets.get(result["native_id"], 0), result["attempt"] + 1)
            for result in parsed:
                result["attempt"] += offsets.get(result["native_id"], 0)
            results.extend(parsed)
        elif document["format"] == "playwright-json" and native["framework"] == "playwright":
            results.extend(parse_playwright(document["content"]))
        else:
            raise ValueError("native parser differs from declared framework")
    return results
