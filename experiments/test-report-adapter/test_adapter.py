"""Synthetic native imports and independent real HTTP publishing."""
from copy import deepcopy
from datetime import datetime, timezone
import hashlib
import http.server
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import threading
import unittest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "packages/python"))
from azimuth_run import fingerprint
from azimuth_testing import normalize_report, partition_report, parse_trx, parse_playwright, load_json
from azimuth_testing.configuration import configuration
from azimuth_testing.publisher import publish_bundle
from azimuth_testing.normalizer import activity_identity

CLI = ROOT / "tools/azimuth/target/debug/azimuth"
DIGEST = "sha256:" + "a" * 64


def fixture(context=False):
    target = {"kind": "artifact", "artifacts": [{"id": "image", "digest": DIGEST}]}
    target_fp = fingerprint({"format": "azimuth-verification-target-fingerprint", "version": 2, "verification_target": target})
    value = {"contract_revision": DIGEST, "values": {"enabled": True}}
    contexts = {"context/component/enabled/activity/" + activity_identity("behavior", "first", 0): fingerprint(True)} if context else {}
    check = {"id": "behavior", "fingerprint": DIGEST, "scenarios": ["works"],
             "implementations": [{"identity": "component|python-symbol|tests:behavior", "source_fingerprint": DIGEST}],
             "units": [{"id": "first", "parameters": {}}]}
    plan = {"model_fingerprint": DIGEST, "required_execution_context": contexts, "checks": [check], "challenges": []}
    plan["fingerprint"] = fingerprint(dict(format="azimuth-run-plan-fingerprint", version=2, verification_target_fingerprint=target_fp, **plan))
    adapter = {"id": "native", "adapter_version": "1", "adapter_fingerprint": DIGEST, "configuration_fingerprint": DIGEST, "descriptor_fingerprint": DIGEST}
    launch = {"operation": "import", "planned_at_ms": 10, "verification_target": target, "verification_target_fingerprint": target_fp,
              "plan": plan, "adapter": adapter, "routes": [{"selection": {"kind": "check", "id": "behavior"},
                  "capability": {"address": "native/reports", "class": "check.import", "fingerprint": DIGEST}}]}
    launch["fingerprint"] = fingerprint(dict(format="azimuth-run-launch-fingerprint", version=2, **launch))
    launch["adapters"] = [launch.pop("adapter")]
    snapshots = []
    if context:
        report = {"format": "azimuth-execution-context", "version": 2, "contexts": {"component": value}}
        snapshots = [{"name": "component", "before": {"captured_at_ms": 21, "instance": "process-1", "report": report},
                      "after": {"captured_at_ms": 29, "instance": "process-1", "report": deepcopy(report)}}]
    report = {"format": "azimuth-test-report", "version": 1, "framework": "xunit-v2", "execution": "execution-1",
              "started_at_ms": 20, "finished_at_ms": 30, "tests": [{"id": "explicit", "attempt": 0, "outcome": "passed",
                  "started_at_ms": 20, "finished_at_ms": 30, "contexts": snapshots, "diagnostics": []}]}
    mapping = {"format": "azimuth-test-mapping", "version": 2, "framework": "xunit-v2", "source_system": "synthetic",
               "tests": [{"id": "explicit", "native_id": "Tests.Behavior|Behavior", "check": "behavior", "unit": "first",
                          "scenarios": ["works"], "components": ["component"], "contexts": [{"component": "component", "leaves": ["enabled"]}] if context else [], "failed_outcome": "inconclusive"}]}
    native = [{"native_id": "Tests.Behavior|Behavior", "attempt": 0, "outcome": "passed", "started_at_ms": 20, "finished_at_ms": 30}]
    arguments = {"native_results": native, "import_inputs": [{"id": "report", "digest": DIGEST, "size_bytes": 1}], "generated_at_ms": 40}
    return launch, report, mapping, arguments


def refresh_launch(launch):
    plan = {key: value for key, value in launch["plan"].items() if key != "fingerprint"}
    launch["plan"]["fingerprint"] = fingerprint(dict(format="azimuth-run-plan-fingerprint", version=2,
        verification_target_fingerprint=launch["verification_target_fingerprint"], **plan))
    payload = {key: launch[key] for key in ("operation", "planned_at_ms", "verification_target", "verification_target_fingerprint", "plan", "routes")}
    payload["adapter"] = launch["adapters"][0]
    launch["fingerprint"] = fingerprint(dict(format="azimuth-run-launch-fingerprint", version=2, **payload))


class Normalization(unittest.TestCase):
    def test_valid_run_passes_actual_native_verifier(self):
        launch, report, mapping, args = fixture()
        bundle = normalize_report(launch, report, mapping, **args)["bundle"]
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "bundle.json"
            path.write_text(json.dumps(bundle))
            result = subprocess.run([str(CLI), "run", "verify", "--bundle", str(path)], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertEqual(result.stderr, "")

    def test_real_playwright_harness_and_json_report_reconcile(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            package = ROOT / "packages/typescript-playwright"
            typed_path, native_path = root / "typed.json", root / "native.json"
            config = {"testDir": str(root), "workers": 1, "retries": 1,
                      "reporter": [[str(package / "dist/reporter.js"), {"outputFile": str(typed_path), "execution": "synthetic-playwright"}],
                                   ["json", {"outputFile": str(native_path)}]], "use": {"azimuthContexts": []}}
            (root / "playwright.config.cjs").write_text("module.exports=" + json.dumps(config))
            (root / "behavior.spec.cjs").write_text("const {test,expect}=require(" + json.dumps(str(package / "dist/index.js")) + ");\n" +
                "test('passes',async()=>{expect(2+2).toBe(4)});\n" +
                "test('fails',async()=>{expect(2+2).toBe(5)});\n" +
                "test.skip('skipped',async()=>{});\n")
            result = subprocess.run(["node", str(package / "node_modules/@playwright/test/cli.js"), "test", "--config", str(root / "playwright.config.cjs")],
                                    cwd=package, capture_output=True, text=True)
            self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
            self.assertEqual(result.stderr, "")
            report = load_json(typed_path.read_bytes())
            native_document = load_json(native_path.read_bytes())
            native = parse_playwright(native_document)
            self.assertEqual(len(report["tests"]), 4)
            self.assertEqual(len(native), 4)
            launch, _, mapping, args = fixture()
            launch["planned_at_ms"] = report["started_at_ms"] - 1
            payload = {key: launch[key] for key in ("operation", "planned_at_ms", "verification_target", "verification_target_fingerprint", "plan", "routes")}
            payload["adapter"] = launch["adapters"][0]
            launch["fingerprint"] = fingerprint(dict(format="azimuth-run-launch-fingerprint", version=2, **payload))
            mapping["framework"] = "playwright"
            mapping["tests"] = []
            for identity in sorted({fact["id"] for fact in report["tests"]}):
                native_ids = {item["native_id"] for item in native if item["native_id"].split("|")[0] == identity}
                self.assertEqual(len(native_ids), 1)
                mapping["tests"].append({"id": identity, "native_id": native_ids.pop(), "check": "behavior", "unit": "first",
                                        "scenarios": ["works"], "components": ["component"], "contexts": [], "failed_outcome": "inconclusive"})
            args.update(native_results=native, generated_at_ms=max(report["finished_at_ms"], max(item["finished_at_ms"] for item in native)))
            bundle = normalize_report(launch, report, mapping, **args)["bundle"]
            self.assertEqual(bundle["check_executions"][0]["results"][0]["outcome"], "inconclusive")
            self.assertEqual(len(bundle["check_executions"][0]["units"][0]["attempts"]), 2)

    def test_real_xunit_harness_report_reconciles_with_actual_trx(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            report_path = root / "report.json"
            import os
            environment = dict(os.environ, AZIMUTH_SYNTHETIC_REPORT=str(report_path))
            result = subprocess.run(["dotnet", "test", str(ROOT / "packages/dotnet/Azimuth.Testing.Xunit.Tests/SyntheticRunner/SyntheticRunner.csproj"),
                                     "--nologo", "--logger", "trx", "--results-directory", str(root / "results")],
                                    env=environment, capture_output=True, text=True)
            self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
            self.assertIn("expected native runner failure", result.stdout)
            self.assertIn("RunnerTests.FailingTest [FAIL]", result.stderr)
            report = load_json(report_path.read_bytes())
            native = parse_trx(next((root / "results").glob("*.trx")).read_bytes())
            self.assertEqual(len(native), 3)
            launch, _, mapping, args = fixture()
            launch["planned_at_ms"] = report["started_at_ms"] - 1
            payload = {key: launch[key] for key in ("operation", "planned_at_ms", "verification_target", "verification_target_fingerprint", "plan", "routes")}
            payload["adapter"] = launch["adapters"][0]
            launch["fingerprint"] = fingerprint(dict(format="azimuth-run-launch-fingerprint", version=2, **payload))
            stem = "Azimuth.Testing.Xunit.SyntheticRunner.RunnerTests."
            mapping["tests"] = []
            for identity, method in (("synthetic.failing", "FailingTest"), ("synthetic.passing", "PassingTest"), ("synthetic.skipped", "SkippedTest")):
                mapping["tests"].append({"id": identity, "native_id": stem + method + "|" + stem + method,
                    "check": "behavior", "unit": "first", "scenarios": ["works"], "components": ["component"], "contexts": [], "failed_outcome": "inconclusive"})
            args.update(native_results=native, generated_at_ms=max(report["finished_at_ms"], max(item["finished_at_ms"] for item in native)))
            bundle = normalize_report(launch, report, mapping, **args)["bundle"]
            self.assertEqual(bundle["check_executions"][0]["results"][0]["outcome"], "inconclusive")
            self.assertIn("native-test-not-captured", [item["message"] for item in bundle["diagnostics"]])

    def test_real_cli_description_plan_and_import(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            spec = importlib.util.spec_from_file_location("synthetic_capabilities", ROOT / "experiments/adapter-capabilities/generate.py")
            generator = importlib.util.module_from_spec(spec); spec.loader.exec_module(generator)
            generator.initialize(root / "fixture")
            fixture_root = root / "fixture"
            module = fixture_root / "model/synthetic"
            nested = module / "behavior"; nested.mkdir()
            for path in list(module.iterdir()):
                if path.is_file(): path.rename(nested / path.name)
            config = root / "adapters.json"; config.write_text(json.dumps(configuration()))
            request = json.loads((fixture_root / "request-import.json").read_text())
            request["checks"][0]["execution-capability"] = "azimuth-tests/reports"
            request_path = root / "request.json"; request_path.write_text(json.dumps(request))
            def execute(arguments):
                result = subprocess.run([str(CLI), *arguments], capture_output=True, text=True)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                self.assertEqual(result.stderr, "")
                return result.stdout
            execute(["adapter", "verify", "--config", str(config)])
            launch_path = root / "launch.json"
            execute(["run", "plan", "--request", str(request_path), "--model", str(fixture_root / "model"),
                     "--standards", str(fixture_root / "standards.md"), "--workspace", str(fixture_root / "workspace.json"),
                     "--manifest", str(fixture_root / "manifest.json"), "--config", str(config), "--out", str(launch_path)])
            launch = load_json(launch_path.read_bytes())
            start, finish = request["planned_at_ms"] + 10, request["planned_at_ms"] + 20
            report = {"format": "azimuth-test-report", "version": 1, "framework": "xunit-v2", "execution": "real-import",
                      "started_at_ms": start, "finished_at_ms": finish, "tests": [{"id": "explicit", "attempt": 0,
                          "outcome": "passed", "started_at_ms": start, "finished_at_ms": finish, "contexts": [], "diagnostics": []}]}
            mapping = {"format": "azimuth-test-mapping", "version": 2, "framework": "xunit-v2", "source_system": "synthetic",
                       "tests": [{"id": "explicit", "native_id": "Tests.Behavior|Behavior", "check": "behavior-check", "unit": "whole",
                                  "scenarios": ["works"], "components": ["component"], "contexts": [], "failed_outcome": "inconclusive"}]}
            iso = lambda value: datetime.fromtimestamp(value / 1000, timezone.utc).isoformat()
            trx = '<TestRun><TestDefinitions><UnitTest id="id"><TestMethod className="Tests" name="Behavior"/></UnitTest></TestDefinitions><Results><UnitTestResult executionId="execution" testId="id" testName="Behavior" outcome="Passed" startTime="' + iso(start) + '" endTime="' + iso(finish) + '"/></Results></TestRun>'
            native = {"format": "azimuth-native-test-results", "version": 1, "framework": "xunit-v2", "documents": [{"format": "trx", "content": trx}]}
            arguments = ["run", "import", "--plan", str(launch_path), "--config", str(config)]
            for identity, value in (("mapping", mapping), ("native-results", native), ("report", report)):
                path = root / (identity + ".json"); path.write_text(json.dumps(value))
                arguments += ["--input", identity + "=" + str(path)]
            bundle_path = root / "bundle.json"
            execute(arguments + ["--out", str(bundle_path)])
            execute(["run", "verify", "--bundle", str(bundle_path)])
            bundle = load_json(bundle_path.read_bytes())
            self.assertEqual(bundle["check_executions"][0]["results"][0]["outcome"], "satisfied")

    def test_failure_is_inconclusive_unless_explicitly_reviewed(self):
        launch, report, mapping, args = fixture()
        report["tests"][0]["outcome"] = args["native_results"][0]["outcome"] = "failed"
        bundle = normalize_report(launch, report, mapping, **args)["bundle"]
        self.assertEqual(bundle["check_executions"][0]["results"][0]["outcome"], "inconclusive")
        mapping["tests"][0]["failed_outcome"] = "violated"
        bundle = normalize_report(launch, report, mapping, **args)["bundle"]
        self.assertEqual(bundle["check_executions"][0]["results"][0]["outcome"], "violated")

    def test_missing_typed_facts_and_static_native_skips_are_not_success(self):
        launch, report, mapping, args = fixture()
        report["tests"] = []
        for outcome in ("skipped", "passed", "failed"):
            args["native_results"][0]["outcome"] = outcome
            bundle = normalize_report(launch, report, mapping, **args)["bundle"]
            self.assertEqual(bundle["check_executions"][0]["results"][0]["outcome"], "inconclusive")
            self.assertIn("native-test-not-captured", [item["message"] for item in bundle["diagnostics"]])

    def test_mapped_missing_native_unit_produces_diagnostic(self):
        launch, report, mapping, args = fixture()
        report["tests"] = []
        args["native_results"] = []
        bundle = normalize_report(launch, report, mapping, **args)["bundle"]
        self.assertEqual(bundle["diagnostics"][0]["code"], "native-unit-not-covered")

    def test_retry_preserves_declared_adverse_first_attempt(self):
        launch, report, mapping, args = fixture()
        mapping["tests"][0]["failed_outcome"] = "violated"
        report["tests"][0]["outcome"] = args["native_results"][0]["outcome"] = "failed"
        retry = deepcopy(report["tests"][0]); retry.update(attempt=1, outcome="passed")
        report["tests"].append(retry)
        native = deepcopy(args["native_results"][0]); native.update(attempt=1, outcome="passed")
        args["native_results"].append(native)
        execution = normalize_report(launch, report, mapping, **args)["bundle"]["check_executions"][0]
        self.assertEqual(len(execution["units"][0]["attempts"]), 2)
        self.assertEqual(execution["results"][0]["outcome"], "violated")

    def test_context_sidecar_pins_exact_run_without_asserting_trust(self):
        launch, report, mapping, args = fixture(context=True)
        result = normalize_report(launch, report, mapping, **args)
        capture = result["context_captures"][0]
        self.assertNotIn("trusted", capture)
        self.assertEqual(capture["run"], result["bundle"]["run_id"])
        self.assertEqual(capture["bundle_fingerprint"], result["bundle"]["bundle_fingerprint"])
        self.assertEqual((result["bundle"]["started_at_ms"], result["bundle"]["finished_at_ms"]), (20, 30))
        self.assertEqual(partition_report(report, mapping, native_results=args["native_results"])[0]["context"], launch["plan"]["required_execution_context"])

    def test_multiple_context_attempts_pin_independent_activity_contexts(self):
        launch, report, mapping, args = fixture(context=True)
        report["tests"].append(dict(deepcopy(report["tests"][0]), attempt=1))
        args["native_results"].append(dict(deepcopy(args["native_results"][0]), attempt=1))
        key = "context/component/enabled/activity/" + activity_identity("behavior", "first", 1)
        launch["plan"]["required_execution_context"][key] = next(iter(launch["plan"]["required_execution_context"].values()))
        refresh_launch(launch)
        result = normalize_report(launch, report, mapping, **args)
        self.assertEqual(len(result["context_captures"]), 2)
        self.assertEqual(len({capture["activity"] for capture in result["context_captures"]}), 2)
        del launch["plan"]["required_execution_context"][key]
        refresh_launch(launch)
        result = normalize_report(launch, report, mapping, **args, selection=[{"id": "explicit", "attempt": 1}])
        self.assertIn("/partition/", result["bundle"]["provenance"]["source"]["execution"])

    def test_disjoint_fixture_units_preserve_each_activity_context_and_verify(self):
        launch, report, mapping, args = fixture(context=True)
        second = deepcopy(report["tests"][0])
        second.update(id="second", started_at_ms=31, finished_at_ms=39)
        for key, time in (("before", 32), ("after", 38)):
            snapshot = second["contexts"][0][key]
            snapshot.update(captured_at_ms=time, instance="second-process")
            snapshot["report"]["contexts"]["component"]["values"]["enabled"] = False
        report["tests"].append(second)
        report["finished_at_ms"] = 39
        mapped = deepcopy(mapping["tests"][0])
        mapped.update(id="second", native_id="Tests.Second|Second", unit="second")
        mapping["tests"].append(mapped)
        args["native_results"].append(dict(native_id="Tests.Second|Second", attempt=0, outcome="passed", started_at_ms=31, finished_at_ms=39))
        launch["plan"]["checks"][0]["units"].append({"id": "second", "parameters": {}})
        activity = activity_identity("behavior", "second", 0)
        value = second["contexts"][0]["before"]["report"]["contexts"]["component"]["values"]["enabled"]
        launch["plan"]["required_execution_context"]["context/component/enabled/activity/" + activity] = fingerprint(value)
        refresh_launch(launch)
        result = normalize_report(launch, report, mapping, **args)
        self.assertEqual(len(result["context_captures"]), 2)
        self.assertEqual({(capture["started_at_ms"], capture["finished_at_ms"]) for capture in result["context_captures"]}, {(21, 29), (32, 38)})
        self.assertEqual(result["bundle"]["check_executions"][0]["results"][0]["outcome"], "satisfied")
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "bundle.json"
            path.write_text(json.dumps(result["bundle"]))
            verified = subprocess.run([str(CLI), "run", "verify", "--bundle", str(path)], capture_output=True, text=True)
            self.assertEqual(verified.returncode, 0, verified.stdout + verified.stderr)
            self.assertEqual(verified.stderr, "")

    def test_changed_context_is_inconclusive_not_a_product_violation(self):
        launch, report, mapping, args = fixture(context=True)
        report["tests"][0]["contexts"][0]["after"]["instance"] = "replacement"
        result = normalize_report(launch, report, mapping, **args)
        self.assertEqual(result["bundle"]["check_executions"][0]["results"][0]["outcome"], "inconclusive")
        self.assertEqual(result["context_captures"], [])

    def test_unknown_fields_duplicate_keys_and_native_disagreement_fail_closed(self):
        launch, report, mapping, args = fixture()
        report["extra"] = True
        with self.assertRaises(ValueError): normalize_report(launch, report, mapping, **args)
        with self.assertRaises(ValueError): load_json('{"a":1,"a":2}')
        del report["extra"]
        args["native_results"][0]["outcome"] = "failed"
        with self.assertRaisesRegex(ValueError, "differs from native"):
            normalize_report(launch, report, mapping, **args)

    def test_trx_and_playwright_native_parsers_preserve_identity_and_skip(self):
        xml = '<TestRun><TestDefinitions><UnitTest id="id"><TestMethod className="Tests" name="Behavior"/></UnitTest></TestDefinitions><Results><UnitTestResult executionId="execution" testId="id" testName="Behavior" outcome="NotExecuted" startTime="2026-01-01T00:00:00Z" endTime="2026-01-01T00:00:01Z"/></Results></TestRun>'
        self.assertEqual(parse_trx(xml)[0]["native_id"], "Tests.Behavior|Behavior")
        self.assertEqual(parse_trx(xml)[0]["outcome"], "skipped")
        precise = xml.replace("2026-01-01T00:00:00Z", "2026-01-01T05:00:00.1234567+05:00")
        self.assertEqual(parse_trx(precise)[0]["started_at_ms"], 1767225600123)
        report = {"suites": [{"specs": [{"id": "native", "tests": [{"projectId": "chromium", "results": [{"retry": 1, "status": "timedOut", "startTime": "2026-01-01T00:00:00Z", "duration": 1.2}]}]}]}]}
        self.assertEqual(parse_playwright(report)[0], {"native_id": "native|chromium", "attempt": 1, "outcome": "timed-out", "started_at_ms": 1767225600000, "finished_at_ms": 1767225600002})


class Publishing(unittest.TestCase):
    def test_real_http_retries_exact_bytes_without_test_execution(self):
        launch, report, mapping, args = fixture()
        bundle = normalize_report(launch, report, mapping, **args)["bundle"]
        content = json.dumps(bundle).encode()
        received = []
        class Handler(http.server.BaseHTTPRequestHandler):
            def log_message(self, *_): pass
            def do_POST(self):
                received.append((self.path, self.rfile.read(int(self.headers["Content-Length"])), self.headers["Authorization"]))
                if len(received) == 1:
                    self.send_response(503); self.end_headers(); return
                self.send_response(201); self.end_headers()
                self.wfile.write(json.dumps({"run_id": bundle["run_id"], "bundle_fingerprint": bundle["bundle_fingerprint"]}).encode())
        server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        thread = threading.Thread(target=server.serve_forever); thread.start()
        try:
            receipt = publish_bundle(bundle_bytes=content, cli=CLI, base_url="http://127.0.0.1:" + str(server.server_port), project="synthetic", token="synthetic-only-token", retry_delay_seconds=0, allow_local_http=True)
        finally:
            server.shutdown(); server.server_close(); thread.join()
        self.assertEqual(receipt["bundle_fingerprint"], bundle["bundle_fingerprint"])
        self.assertEqual(len(received), 2)
        self.assertTrue(all(item[1] == content for item in received))
        self.assertTrue(all(item[0] == "/v1/projects/synthetic/runs" for item in received))

    def test_conflict_is_terminal_and_invalid_bundle_is_never_uploaded(self):
        launch, report, mapping, args = fixture()
        bundle = normalize_report(launch, report, mapping, **args)["bundle"]
        received = []
        class Handler(http.server.BaseHTTPRequestHandler):
            def log_message(self, *_): pass
            def do_POST(self):
                received.append(self.rfile.read(int(self.headers["Content-Length"])))
                self.send_response(409); self.end_headers()
        server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        thread = threading.Thread(target=server.serve_forever); thread.start()
        arguments = dict(cli=CLI, base_url="http://127.0.0.1:" + str(server.server_port), project="synthetic", token="synthetic-only-token", retry_delay_seconds=0, allow_local_http=True)
        try:
            with self.assertRaisesRegex(ValueError, "409"):
                publish_bundle(bundle_bytes=json.dumps(bundle).encode(), **arguments)
            with self.assertRaisesRegex(ValueError, "core rejected"):
                publish_bundle(bundle_bytes=b'{"invalid":true}', **arguments)
        finally:
            server.shutdown(); server.server_close(); thread.join()
        self.assertEqual(len(received), 1)


if __name__ == "__main__": unittest.main()
