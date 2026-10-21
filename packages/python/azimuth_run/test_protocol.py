"""Run construction regressions independent of provider outcome interpretation."""
from copy import deepcopy
import unittest

from azimuth_run import build_check_bundle, canonical_json, fingerprint


def fixture():
    digest = "sha256:" + "a" * 64
    target = {"kind": "artifact", "artifacts": [{"id": "image", "digest": digest}]}
    target_fingerprint = fingerprint({"format": "azimuth-verification-target-fingerprint", "version": 2,
                                      "verification_target": target})
    checks = [{"id": "admission", "fingerprint": digest, "scenarios": ["denied"],
               "implementations": [{"identity": "api|python-symbol|checks:admission", "source_fingerprint": digest}],
               "units": [{"id": "first", "parameters": {}}, {"id": "second", "parameters": {}}]}]
    plan = {"model_fingerprint": digest, "required_execution_context": {}, "checks": checks, "challenges": []}
    plan["fingerprint"] = fingerprint({"format": "azimuth-run-plan-fingerprint", "version": 2,
                                      "verification_target_fingerprint": target_fingerprint, **plan})
    launch = {"operation": "import", "planned_at_ms": 10, "verification_target": target,
              "verification_target_fingerprint": target_fingerprint, "plan": plan,
              "adapter": {"id": "native", "adapter_version": "1", "adapter_fingerprint": digest,
                          "configuration_fingerprint": digest, "descriptor_fingerprint": digest},
              "routes": [{"selection": {"kind": "check", "id": "admission"},
                          "capability": {"address": "native/reports", "class": "check.import", "fingerprint": digest}}]}
    launch["fingerprint"] = fingerprint({"format": "azimuth-run-launch-fingerprint", "version": 2, **launch})
    launch["adapters"] = [launch.pop("adapter")]
    activities = [{"id": identity, "status": "completed", "started_at_ms": 20,
                   "finished_at_ms": 30, "artifacts": [], "diagnostics": [], "attributes": {}}
                  for identity in ("measurement-a", "measurement-b")]
    units = [{"id": identity, "attempts": [{"ordinal": 0, "activity": activity, "outcomes": {"denied": "satisfied"}}]}
             for identity, activity in zip(("first", "second"), ("measurement-a", "measurement-b"))]
    arguments = {"source": {"system": "native", "execution": "execution-1"},
                 "normalizer": {"id": "adapter/native", "version": "1", "build_fingerprint": digest},
                 "generated_at_ms": 40, "started_at_ms": 20, "finished_at_ms": 30, "actual_context": {},
                 "activities": activities, "units_by_check": {"admission": units}, "artifacts": [], "diagnostics": [],
                 "import_inputs": [{"id": "native-account", "digest": digest, "size_bytes": 10}]}
    return launch, arguments


class RunConstruction(unittest.TestCase):
    def test_exact_native_interval_and_configuration_are_retained(self):
        launch, arguments = fixture()
        bundle = build_check_bundle(launch, **arguments)
        self.assertEqual(bundle["verification_target"], launch["verification_target"])
        self.assertEqual((bundle["planned_at_ms"], bundle["started_at_ms"], bundle["finished_at_ms"]), (10, 20, 30))
        self.assertEqual(bundle["provenance"]["normalizer"], arguments["normalizer"])
        self.assertEqual(bundle["check_executions"][0]["results"][0]["outcome"], "satisfied")
        self.assertEqual(bundle["check_executions"][0]["units"][0]["attempts"][0]["ordinal"], 1)
        self.assertEqual(arguments["units_by_check"]["admission"][0]["attempts"][0]["ordinal"], 0)
        self.assertEqual(launch, fixture()[0])

    def test_bare_adapter_identity_and_substituted_normalizer_are_rejected(self):
        for field, value in (("id", "native"), ("version", "2"), ("build_fingerprint", "sha256:" + "b" * 64)):
            with self.subTest(field=field):
                launch, arguments = fixture()
                arguments["normalizer"][field] = value
                with self.assertRaisesRegex(ValueError, "normalizer must exactly identify"):
                    build_check_bundle(launch, **arguments)

    def test_missing_unit_remains_visible_and_inconclusive(self):
        launch, arguments = fixture()
        arguments["units_by_check"]["admission"].pop()
        bundle = build_check_bundle(launch, **arguments)
        execution = bundle["check_executions"][0]
        self.assertEqual(execution["results"][0]["outcome"], "inconclusive")
        self.assertEqual(len(execution["units"]), 2)
        self.assertEqual(bundle["diagnostics"][0]["code"], "native-unit-not-covered")
        self.assertEqual(execution["units"][1]["attempts"][0]["outcomes"], {"denied": "inconclusive"})
        self.assertEqual(execution["units"][1]["attempts"][0]["ordinal"], 1)

    def test_violation_is_not_erased_by_a_successful_retry(self):
        launch, arguments = fixture()
        first = arguments["units_by_check"]["admission"][0]["attempts"][0]
        first["outcomes"]["denied"] = "violated"
        retry = deepcopy(arguments["activities"][0]); retry["id"] = "retry"
        arguments["activities"].append(retry)
        arguments["units_by_check"]["admission"][0]["attempts"].append(
            {"ordinal": 1, "activity": "retry", "outcomes": {"denied": "satisfied"}})
        bundle = build_check_bundle(launch, **arguments)
        self.assertEqual(bundle["check_executions"][0]["results"][0]["outcome"], "violated")

    def test_unplanned_check_unit_and_scenario_are_rejected(self):
        for domain in ("check", "unit", "scenario"):
            with self.subTest(domain=domain):
                launch, arguments = fixture()
                if domain == "check": arguments["units_by_check"]["substitute"] = []
                if domain == "unit": arguments["units_by_check"]["admission"][0]["id"] = "substitute"
                if domain == "scenario": arguments["units_by_check"]["admission"][0]["attempts"][0]["outcomes"] = {"substitute": "satisfied"}
                with self.assertRaises(ValueError): build_check_bundle(launch, **arguments)

    def test_failed_measurement_cannot_establish_a_proposition(self):
        launch, arguments = fixture()
        arguments["activities"][0]["status"] = "failed"
        with self.assertRaisesRegex(ValueError, "failed native activity"):
            build_check_bundle(launch, **arguments)

    def test_measurement_before_launch_is_rejected(self):
        launch, arguments = fixture()
        arguments["started_at_ms"] = 9
        with self.assertRaisesRegex(ValueError, "native execution time interval"):
            build_check_bundle(launch, **arguments)

    def test_correction_preserves_native_start_and_source(self):
        launch, arguments = fixture()
        first = build_check_bundle(launch, **arguments)
        arguments["generated_at_ms"] = 50
        correction = build_check_bundle(launch, **arguments, predecessors=[first], correction_reason="Corrected normalization")
        self.assertEqual(correction["run_id"], first["run_id"])
        self.assertEqual(correction["corrects"], first["bundle_fingerprint"])
        self.assertEqual(correction["bundle_revision"], 1)
        arguments["started_at_ms"] = 19
        with self.assertRaisesRegex(ValueError, "execution anchor"):
            build_check_bundle(launch, **arguments, predecessors=[first], correction_reason="Incorrect start")

    def test_canonicalization_uses_utf16_order_and_rejects_unsafe_numbers(self):
        self.assertEqual(canonical_json({"\ue000": 1, "\U00010000": 2}), '{"\U00010000":2,"\ue000":1}')
        for value in (-1, 9007199254740992, 1.0, float("nan"), "\ud800"):
            with self.subTest(value=repr(value)):
                with self.assertRaises((ValueError, UnicodeError)): canonical_json(value)


class PartitionedLaunchTests(unittest.TestCase):
    def test_provider_builder_rejects_a_coordinated_parent_launch(self):
        launch, arguments = fixture()
        launch["adapters"].append(deepcopy(launch["adapters"][0]))
        with self.assertRaisesRegex(ValueError, "partitioned adapter launch"):
            build_check_bundle(launch, **arguments)


class RustFrozenVectorParity(unittest.TestCase):
    """Vectors frozen in tools/azimuth/tests/run.rs and run_plan.rs; the Python mirror must match byte-for-byte."""

    TARGET = {"kind": "artifact", "artifacts": [{"id": "image", "digest": "sha256:" + "4" * 64}]}
    TARGET_FINGERPRINT = "sha256:339156da3a55df33136e71859f1a3482bb39ee9aa45c11a30780334ee9e63759"
    PLAN_FINGERPRINT = "sha256:b75606956b9c1857f8b401d9bad207253b90f6948efddb5532a769b9f488fbfb"

    def launch_payload(self, scenarios):
        digest = lambda character: "sha256:" + character * 64
        check = {"id": "check", "fingerprint": digest("6"),
                 "implementations": [{"identity": "demo|rust-symbol|demo::check", "source_fingerprint": digest("7")}],
                 "units": [{"id": "whole", "parameters": {}}]}
        if scenarios is not None:
            check["scenarios"] = scenarios
        return {"format": "azimuth-run-launch-fingerprint", "version": 2, "operation": "execute",
                "planned_at_ms": 1787300000000, "verification_target": self.TARGET,
                "verification_target_fingerprint": self.TARGET_FINGERPRINT,
                "plan": {"model_fingerprint": digest("8"), "required_execution_context": {}, "checks": [check],
                         "challenges": [], "fingerprint": self.PLAN_FINGERPRINT},
                "adapter": {"id": "demo", "adapter_version": "1", "adapter_fingerprint": digest("0"),
                            "configuration_fingerprint": digest("1"), "descriptor_fingerprint": digest("2")},
                "routes": [{"selection": {"kind": "check", "id": "check"},
                            "capability": {"address": "demo/check", "class": "check.execute",
                                           "fingerprint": digest("3")}}]}

    def test_verification_target_fingerprint(self):
        self.assertEqual(fingerprint({"format": "azimuth-verification-target-fingerprint", "version": 2,
                                      "verification_target": self.TARGET}), self.TARGET_FINGERPRINT)

    def test_run_identity(self):
        self.assertEqual(fingerprint({
            "format": "azimuth-run-identity", "version": 2, "source_system": "synthetic", "source_execution": "run-1",
            "verification_target_fingerprint": self.TARGET_FINGERPRINT, "plan_fingerprint": self.PLAN_FINGERPRINT,
            "launch_fingerprint": "sha256:980dc9e544f41414e3a2735e84a6d9733aee85b2961899bb538f1f34c4347237"}),
            "sha256:2bf29414172cc5dffce83955f95be42a9d56fd3ed098a25f6c7cc070547395f5")

    def test_launch_fingerprint(self):
        self.assertEqual(fingerprint(self.launch_payload(["works"])),
                         "sha256:efe730885be229f75122c7e69a69fb931d6abfe7587336ad47f790fd03320179")
        self.assertEqual(fingerprint(self.launch_payload(None)),
                         "sha256:1fed0abd20f4df6e7ba522b0cab896d241bf7d625f23a1a8eb077027d6cb89b7")


if __name__ == "__main__":
    unittest.main()
