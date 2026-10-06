"""Run construction regressions independent of provider outcome interpretation."""
from copy import deepcopy
import unittest

from azimuth_run import build_check_bundle, canonical_json, fingerprint


def fixture():
    digest = "sha256:" + "a" * 64
    subject = {"kind": "artifact", "artifacts": [{"id": "image", "digest": digest}]}
    subject_fingerprint = fingerprint({"format": "azimuth-subject-fingerprint", "version": 1, "subject": subject})
    checks = [{"id": "admission", "fingerprint": digest, "cases": ["denied"],
               "implementations": [{"identity": "api|python-symbol|checks:admission", "source_fingerprint": digest}],
               "units": [{"id": "first", "parameters": {}}, {"id": "second", "parameters": {}}]}]
    plan = {"model_fingerprint": digest, "required_context": {}, "checks": checks, "challenges": []}
    plan["fingerprint"] = fingerprint({"format": "azimuth-run-plan-fingerprint", "version": 1,
                                      "subject_fingerprint": subject_fingerprint, **plan})
    launch = {"operation": "import", "planned_at_ms": 10, "subject": subject,
              "subject_fingerprint": subject_fingerprint, "plan": plan,
              "adapter": {"id": "native", "adapter_version": "1", "adapter_fingerprint": digest,
                          "configuration_fingerprint": digest, "descriptor_fingerprint": digest},
              "routes": [{"selection": {"kind": "check", "id": "admission"},
                          "capability": {"address": "native/reports", "class": "check.import", "fingerprint": digest}}]}
    launch["fingerprint"] = fingerprint({"format": "azimuth-run-launch-fingerprint", "version": 1, **launch})
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
        self.assertEqual(bundle["subject"], launch["subject"])
        self.assertEqual((bundle["planned_at_ms"], bundle["started_at_ms"], bundle["finished_at_ms"]), (10, 20, 30))
        self.assertEqual(bundle["provenance"]["normalizer"], arguments["normalizer"])
        self.assertEqual(bundle["check_executions"][0]["observations"][0]["outcome"], "satisfied")
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
        self.assertEqual(execution["observations"][0]["outcome"], "inconclusive")
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
        self.assertEqual(bundle["check_executions"][0]["observations"][0]["outcome"], "violated")

    def test_unplanned_check_unit_and_case_are_rejected(self):
        for domain in ("check", "unit", "case"):
            with self.subTest(domain=domain):
                launch, arguments = fixture()
                if domain == "check": arguments["units_by_check"]["substitute"] = []
                if domain == "unit": arguments["units_by_check"]["admission"][0]["id"] = "substitute"
                if domain == "case": arguments["units_by_check"]["admission"][0]["attempts"][0]["outcomes"] = {"substitute": "satisfied"}
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


if __name__ == "__main__":
    unittest.main()
