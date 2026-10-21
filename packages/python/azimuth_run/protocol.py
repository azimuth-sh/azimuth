"""Protocol construction; provider meaning remains with the native normalizer."""
from copy import deepcopy
import hashlib
import json

_MAX_INTEGER = 9007199254740991
_OUTCOMES = {"satisfied", "violated", "inconclusive"}


def canonical_json(value):
    """RFC 8785 serialization for the Run protocol's safe integer domain."""
    if value is None or isinstance(value, bool):
        return json.dumps(value, ensure_ascii=False)
    if isinstance(value, int):
        if not 0 <= value <= _MAX_INTEGER:
            raise ValueError("Run numbers must be nonnegative safe integers")
        return str(value)
    if isinstance(value, str):
        value.encode("utf-8", errors="strict")
        return json.dumps(value, ensure_ascii=False, separators=(",", ":"))
    if isinstance(value, list):
        return "[" + ",".join(canonical_json(item) for item in value) + "]"
    if isinstance(value, dict) and all(isinstance(key, str) for key in value):
        keys = sorted(value, key=lambda key: key.encode("utf-16-be", errors="strict"))
        return "{" + ",".join(canonical_json(key) + ":" + canonical_json(value[key]) for key in keys) + "}"
    raise ValueError("unsupported canonical JSON value")


def fingerprint(value):
    return "sha256:" + hashlib.sha256(canonical_json(value).encode("utf-8")).hexdigest()


def _identity_map(items, label):
    result = {}
    for item in items:
        identity = item["id"]
        if not isinstance(identity, str) or not identity or identity in result:
            raise ValueError(f"{label} identities must be unique nonempty strings")
        result[identity] = deepcopy(item)
    return result


def build_check_bundle(launch, *, source, normalizer, generated_at_ms,
                       started_at_ms, finished_at_ms, actual_context,
                       activities, units_by_check, artifacts, diagnostics,
                       import_inputs, predecessors=(), correction_reason=None):
    """Construct a Check-only import; Azimuth's Run verifier is the final gate.

    units_by_check maps Check IDs to {id, attempts} records. Every attempt has
    ordinal, activity and an outcomes map covering exactly the selected Scenarios.
    Missing planned units receive explicit inconclusive attempts, never success.
    predecessors contains the complete prior wire bundles for this same Run.
    """
    launch = deepcopy(launch)
    if len(launch.get("adapters", [])) != 1:
        raise ValueError("provider bundle builder requires one partitioned adapter launch")
    selected_adapter = launch["adapters"][0]
    expected_normalizer = {"id": "adapter/" + selected_adapter["id"],
                           "version": selected_adapter["adapter_version"],
                           "build_fingerprint": selected_adapter["adapter_fingerprint"]}
    if normalizer != expected_normalizer:
        raise ValueError("normalizer must exactly identify the configured adapter")
    if launch["operation"] != "import" or launch["plan"]["challenges"]:
        raise ValueError("this builder accepts Check-only import launches")
    if not launch["planned_at_ms"] <= started_at_ms <= finished_at_ms <= generated_at_ms:
        raise ValueError("invalid native execution time interval")
    target_fp = fingerprint({"format": "azimuth-verification-target-fingerprint", "version": 2,
                             "verification_target": launch["verification_target"]})
    if target_fp != launch["verification_target_fingerprint"]:
        raise ValueError("Verification Target fingerprint mismatch")
    launch_payload = {key: launch[key] for key in (
        "operation", "planned_at_ms", "verification_target", "verification_target_fingerprint", "plan", "routes")}
    launch_payload["adapter"] = deepcopy(selected_adapter)
    launch_payload.update(format="azimuth-run-launch-fingerprint", version=2)
    if fingerprint(launch_payload) != launch["fingerprint"]:
        raise ValueError("launch fingerprint mismatch")
    checks = launch["plan"]["checks"]
    if set(units_by_check) - {check["id"] for check in checks}:
        raise ValueError("native account names an unplanned Check")
    activity_map = _identity_map(activities, "activity")
    artifact_map = _identity_map(artifacts, "artifact")
    diagnostic_map = _identity_map(diagnostics, "diagnostic")
    for activity in activity_map.values():
        if not started_at_ms <= activity["started_at_ms"] <= activity["finished_at_ms"] <= finished_at_ms:
            raise ValueError("activity lies outside native execution interval")
    adapter = deepcopy(selected_adapter)
    adapter.update(launch_fingerprint=launch["fingerprint"], routes=launch["routes"],
                   import_inputs=sorted(deepcopy(import_inputs), key=lambda item: item["id"]))
    if not adapter["import_inputs"]:
        raise ValueError("imports require accountable input identities")
    selection = {"execution_context": deepcopy(actual_context), "plan_fingerprint": launch["plan"]["fingerprint"],
                 "checks": deepcopy(checks), "challenges": []}
    selection["fingerprint"] = fingerprint(dict(format="azimuth-run-selection-fingerprint", version=2, **selection))
    bundle = {"format": "azimuth-run-bundle", "version": 2, "bundle_revision": len(predecessors),
              "verification_target": launch["verification_target"], "verification_target_fingerprint": target_fp,
              "planned_at_ms": launch["planned_at_ms"], "started_at_ms": started_at_ms,
              "finished_at_ms": finished_at_ms, "status": "complete", "plan": launch["plan"],
              "actual_selection": selection,
              "provenance": {"mode": "import", "source": deepcopy(source), "normalizer": deepcopy(normalizer),
                             "adapter": adapter, "generated_at_ms": generated_at_ms},
              "check_executions": [], "verification_challenge_executions": []}
    bundle["run_id"] = fingerprint({"format": "azimuth-run-identity", "version": 2,
                                   "source_system": source["system"], "source_execution": source["execution"],
                                   "verification_target_fingerprint": target_fp,
                                   "plan_fingerprint": selection["plan_fingerprint"],
                                   "launch_fingerprint": launch["fingerprint"]})
    used_activities = set()
    for check in checks:
        units = _identity_map(units_by_check.get(check["id"], []), "unit")
        if set(units) - {unit["id"] for unit in check["units"]}:
            raise ValueError("native account names an unplanned unit")
        reduced = {scenario: [] for scenario in check["scenarios"]}
        execution = {"check": {"id": check["id"], "fingerprint": check["fingerprint"]},
                     "units": [], "results": []}
        refs, findings = set(), set()
        for planned_unit in check["units"]:
            unit = units.get(planned_unit["id"])
            if unit is None or not unit.get("attempts"):
                suffix = hashlib.sha256((check["id"] + ":" + planned_unit["id"]).encode()).hexdigest()
                activity_id, diagnostic_id = "uncovered-" + suffix, "missing-unit-" + suffix
                if activity_id in activity_map or diagnostic_id in diagnostic_map:
                    raise ValueError("generated incomplete-unit identity collision")
                diagnostic_map[diagnostic_id] = {"id": diagnostic_id, "class": "normalization", "severity": "warning",
                    "code": "native-unit-not-covered", "message": "Native account does not cover a planned unit",
                    "scope": {"kind": "check-execution", "check": check["id"]}, "artifacts": [],
                    "details": {"unit": planned_unit["id"]}}
                activity_map[activity_id] = {"id": activity_id, "status": "failed", "started_at_ms": finished_at_ms,
                    "finished_at_ms": finished_at_ms, "artifacts": [], "diagnostics": [diagnostic_id], "attributes": {}}
                unit = {"id": planned_unit["id"], "attempts": [{"ordinal": 0, "activity": activity_id,
                        "outcomes": {scenario: "inconclusive" for scenario in check["scenarios"]}}]}
            for ordinal, attempt in enumerate(unit["attempts"]):
                if attempt["ordinal"] != ordinal or set(attempt["outcomes"]) != set(check["scenarios"]):
                    raise ValueError("attempt ordinal or Scenario selection mismatch")
                if any(outcome not in _OUTCOMES for outcome in attempt["outcomes"].values()):
                    raise ValueError("unsupported native Scenario outcome")
                activity_id = attempt["activity"]
                if activity_id in used_activities or activity_id not in activity_map:
                    raise ValueError("attempt activities must exist and be unique")
                used_activities.add(activity_id)
                activity = activity_map[activity_id]
                if activity["status"] != "completed" and set(attempt["outcomes"].values()) != {"inconclusive"}:
                    raise ValueError("failed native activity cannot establish a proposition")
                refs.update(activity["artifacts"])
                findings.update(activity["diagnostics"])
            for scenario in check["scenarios"]:
                attempts = unit["attempts"]
                outcome = ("violated" if any(attempt["outcomes"][scenario] == "violated" for attempt in attempts)
                           else attempts[-1]["outcomes"][scenario])
                reduced[scenario].append(outcome)
            wire_unit = deepcopy(unit)
            for attempt in wire_unit["attempts"]:
                attempt["ordinal"] += 1
            execution["units"].append(wire_unit)
        for scenario in check["scenarios"]:
            values = reduced[scenario]
            outcome = "violated" if "violated" in values else "satisfied" if values and set(values) == {"satisfied"} else "inconclusive"
            result = {"scenario": scenario, "outcome": outcome, "observed_at_ms": finished_at_ms,
                      "artifacts": sorted(refs), "diagnostics": sorted(findings)}
            result["fingerprint"] = fingerprint({"format": "azimuth-check-result-fingerprint", "version": 2,
                "run_id": bundle["run_id"], "verification_target_fingerprint": target_fp, "check": execution["check"],
                "scenario": scenario, "execution_context": actual_context, "outcome": outcome,
                "observed_at_ms": finished_at_ms})
            execution["results"].append(result)
        bundle["check_executions"].append(execution)
    bundle.update(activities=sorted(activity_map.values(), key=lambda item: item["id"]),
                  artifacts=sorted(artifact_map.values(), key=lambda item: item["id"]),
                  diagnostics=sorted(diagnostic_map.values(), key=lambda item: item["id"]))
    if predecessors:
        if not correction_reason:
            raise ValueError("corrections require an explicit reason")
        for revision, previous in enumerate(predecessors):
            if previous["run_id"] != bundle["run_id"] or previous["bundle_revision"] != revision:
                raise ValueError("correction history identity mismatch")
            payload = {key: value for key, value in previous.items() if key != "bundle_fingerprint"}
            if fingerprint({"format": "azimuth-run-bundle-fingerprint", "version": 2, "bundle": payload}) != previous["bundle_fingerprint"]:
                raise ValueError("correction predecessor fingerprint mismatch")
            for key in ("verification_target", "verification_target_fingerprint", "planned_at_ms", "started_at_ms", "plan"):
                if previous[key] != bundle[key]:
                    raise ValueError("correction changes an execution anchor")
            if (previous["actual_selection"]["execution_context"] != actual_context
                    or previous["provenance"]["normalizer"] != normalizer):
                raise ValueError("correction changes Execution Context or normalizer anchors")
            previous_adapter = previous["provenance"]["adapter"]
            if any(previous_adapter[key] != adapter[key] for key in adapter if key != "import_inputs"):
                raise ValueError("correction changes adapter anchors")
            if any(previous["provenance"]["source"][key] != source[key] for key in ("system", "execution")):
                raise ValueError("correction changes source execution anchors")
            if revision and previous.get("corrects") != predecessors[revision - 1]["bundle_fingerprint"]:
                raise ValueError("correction predecessor chain mismatch")
        bundle.update(corrects=predecessors[-1]["bundle_fingerprint"], correction_reason=correction_reason)
    elif correction_reason:
        raise ValueError("initial import cannot supply a correction reason")
    bundle["bundle_fingerprint"] = fingerprint({"format": "azimuth-run-bundle-fingerprint", "version": 2, "bundle": bundle})
    return bundle


def build_challenge_bundle(launch, *, source, normalizer, generated_at_ms,
                           started_at_ms, finished_at_ms, actual_context,
                           activities, units_by_challenge, artifacts, diagnostics,
                           import_inputs):
    launch = deepcopy(launch)
    if len(launch.get('adapters', [])) != 1 or launch['operation'] != 'import' or launch['plan']['checks']:
        raise ValueError('Verification Challenge builder requires one partitioned Challenge-only import launch')
    adapter = launch['adapters'][0]
    expected = {'id': 'adapter/' + adapter['id'], 'version': adapter['adapter_version'], 'build_fingerprint': adapter['adapter_fingerprint']}
    if normalizer != expected:
        raise ValueError('normalizer does not identify selected adapter')
    if not launch['planned_at_ms'] <= started_at_ms <= finished_at_ms <= generated_at_ms:
        raise ValueError('invalid execution time interval')
    target_fp = fingerprint({'format':'azimuth-verification-target-fingerprint','version':2,
                             'verification_target':launch['verification_target']})
    if target_fp != launch['verification_target_fingerprint']:
        raise ValueError('Verification Target fingerprint mismatch')
    launch_payload = {key:launch[key] for key in ('operation','planned_at_ms','verification_target',
                                                  'verification_target_fingerprint','plan','routes')}
    launch_payload.update(format='azimuth-run-launch-fingerprint',version=2,adapter=deepcopy(adapter))
    if fingerprint(launch_payload) != launch['fingerprint']:
        raise ValueError('launch fingerprint mismatch')
    activity_map = _identity_map(activities,'activity')
    artifact_map = _identity_map(artifacts,'artifact')
    diagnostic_map = _identity_map(diagnostics,'diagnostic')
    challenges = launch['plan']['challenges']
    if set(units_by_challenge) != {item['id'] for item in challenges}:
        raise ValueError('native Challenge selection must exactly cover plan')
    if not import_inputs:
        raise ValueError('native import requires accountable inputs')
    adapter.update(launch_fingerprint=launch['fingerprint'],routes=launch['routes'],import_inputs=sorted(deepcopy(import_inputs),key=lambda item:item['id']))
    selection = {'execution_context':deepcopy(actual_context),'plan_fingerprint':launch['plan']['fingerprint'],
                 'checks':[],'challenges':deepcopy(challenges)}
    selection['fingerprint'] = fingerprint(dict(format='azimuth-run-selection-fingerprint',version=2,**selection))
    bundle = {'format':'azimuth-run-bundle','version':2,'bundle_revision':0,
              'verification_target':launch['verification_target'],'verification_target_fingerprint':target_fp,'planned_at_ms':launch['planned_at_ms'],'started_at_ms':started_at_ms,
              'finished_at_ms':finished_at_ms,'status':'complete','plan':launch['plan'],'actual_selection':selection,
              'provenance':{'mode':'import','source':deepcopy(source),'normalizer':deepcopy(normalizer),'adapter':adapter,'generated_at_ms':generated_at_ms},
              'check_executions':[],'verification_challenge_executions':[]}
    bundle['run_id'] = fingerprint({'format':'azimuth-run-identity','version':2,'source_system':source['system'],
                                   'source_execution':source['execution'],'verification_target_fingerprint':target_fp,
                                   'plan_fingerprint':selection['plan_fingerprint'],'launch_fingerprint':launch['fingerprint']})
    used = set()
    for challenge in challenges:
        units = _identity_map(units_by_challenge[challenge['id']],'unit')
        if set(units) != {item['id'] for item in challenge['units']}:
            raise ValueError('native Challenge units must exactly cover plan')
        outcomes, objections, refs, diagnostic_refs, wire_units = [], set(), set(), set(), []
        for planned in challenge['units']:
            unit = units[planned['id']]
            attempts = unit.get('attempts', [])
            if not attempts:
                raise ValueError('native Challenge unit has no measured attempt')
            wire_attempts = []
            for ordinal, attempt in enumerate(attempts):
                if attempt['ordinal'] != ordinal or attempt['outcome'] not in {'clean','findings','inconclusive'}:
                    raise ValueError('invalid Challenge attempt')
                activity_id = attempt['activity']
                if activity_id not in activity_map or activity_id in used:
                    raise ValueError('attempt activity must exist and be unique')
                activity = activity_map[activity_id]
                if not started_at_ms <= activity['started_at_ms'] <= activity['finished_at_ms'] <= finished_at_ms:
                    raise ValueError('activity outside execution interval')
                if activity['status'] != 'completed' and attempt['outcome'] != 'inconclusive':
                    raise ValueError('failed activity cannot establish completed search')
                used.add(activity_id)
                refs.update(activity['artifacts'])
                diagnostic_refs.update(activity['diagnostics'])
                objections.update(attempt.get('objections',[]))
                wire_attempts.append({'ordinal':ordinal+1,'activity':activity_id,'outcome':attempt['outcome']})
            outcomes.append('findings' if any(item['outcome']=='findings' for item in attempts) else attempts[-1]['outcome'])
            wire_units.append({'id':unit['id'],'attempts':wire_attempts})
        outcome = 'findings' if 'findings' in outcomes else 'clean' if set(outcomes)=={'clean'} else 'inconclusive'
        if outcome == 'findings' and not objections:
            raise ValueError('findings require concrete objections')
        if outcome == 'clean' and objections:
            raise ValueError('clean result cannot hide objections')
        execution = {'challenge':challenge['id'],'verification_challenge':deepcopy(challenge['verification_challenge']),
                     'target':deepcopy(challenge['target']),'units':wire_units}
        result = {'outcome':outcome,'observed_at_ms':finished_at_ms,'objections':sorted(objections),
                  'artifacts':sorted(refs),'diagnostics':sorted(diagnostic_refs)}
        result['fingerprint'] = fingerprint({'format':'azimuth-challenge-result-fingerprint','version':2,'run_id':bundle['run_id'],
            'challenge':execution['challenge'],'verification_challenge':execution['verification_challenge'],
            'target':execution['target'],
            'outcome':outcome,'observed_at_ms':finished_at_ms})
        execution['result'] = result
        bundle['verification_challenge_executions'].append(execution)
    if used != set(activity_map):
        raise ValueError('native activity not accounted by planned Challenge')
    bundle.update(activities=sorted(activity_map.values(),key=lambda item:item['id']),
                  artifacts=sorted(artifact_map.values(),key=lambda item:item['id']),
                  diagnostics=sorted(diagnostic_map.values(),key=lambda item:item['id']))
    bundle['bundle_fingerprint'] = fingerprint({'format':'azimuth-run-bundle-fingerprint','version':2,'bundle':bundle})
    return bundle
