import copy
import json
import tempfile
import unittest
from pathlib import Path, PurePosixPath

from release.isolate_experiments import (
    IsolationError,
    POLYGLOT_GATE,
    ROOT_GATE,
    WORKFLOW,
    archived_workflow_receipt,
    derive_root_account,
    executable_inputs,
    reference_sources,
    tracked_files,
    validate_domain_inputs,
    validate_domain_references,
    validate_root_sequence,
    validate_workflow,
    validate_workflow_receipt,
)
from release.qualify import catalog_at


class ExperimentalIsolationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        from pathlib import Path

        cls.root = Path(__file__).resolve().parent.parent
        cls.catalog = catalog_at(cls.root)
        cls.tracked = tracked_files(cls.root)

    def test_every_catalog_root_derives_an_executable_gate(self):
        account = derive_root_account(self.catalog, self.root, self.tracked)
        self.assertEqual(
            [item["root"] for item in account],
            sorted(self.catalog["experimentalSource"]),
        )
        self.assertTrue(all(item["gates"] for item in account))

    def test_an_unaccounted_root_fails_with_its_name(self):
        changed = copy.deepcopy(self.catalog)
        changed["experimentalSource"].append("packages/unaccounted")
        with self.assertRaisesRegex(IsolationError, "packages/unaccounted"):
            derive_root_account(changed, self.root, self.tracked)

    def test_removing_an_experiment_gate_fails_with_the_root(self):
        source = (self.root / ROOT_GATE).read_text()
        changed = source.replace("./experiments/polyglot/check.sh", "true")
        with self.assertRaisesRegex(IsolationError, "experiments: root gate"):
            derive_root_account(
                self.catalog,
                self.root,
                self.tracked,
                {ROOT_GATE: changed},
            )

    def test_a_noop_root_mention_is_not_an_executable_relation(self):
        source = (self.root / POLYGLOT_GATE).read_text()
        changed = source.replace("packages/cpp", "packages/not-cpp") + "\necho packages/cpp\n"
        with self.assertRaisesRegex(IsolationError, "packages/cpp"):
            derive_root_account(
                self.catalog,
                self.root,
                self.tracked,
                {POLYGLOT_GATE: changed},
            )

    def test_a_noop_experiment_gate_is_not_coverage(self):
        gate = PurePosixPath("experiments/assurance-service/check.sh")
        with self.assertRaisesRegex(IsolationError, "assurance-service gate"):
            derive_root_account(
                self.catalog,
                self.root,
                self.tracked,
                {gate: "echo experiments/assurance-service\n"},
            )

    def test_reordering_roots_preserves_the_derived_account(self):
        changed = copy.deepcopy(self.catalog)
        changed["experimentalSource"].reverse()
        self.assertEqual(
            derive_root_account(self.catalog, self.root, self.tracked),
            derive_root_account(changed, self.root, self.tracked),
        )

    def test_local_and_mounted_domain_locators_are_rejected(self):
        cases = ("cd ../azimuth-demo", "--root /mnt/drim/project")
        for source in cases:
            with self.subTest(source=source):
                with self.assertRaisesRegex(IsolationError, "external domain locator"):
                    validate_domain_inputs({PurePosixPath("gate.sh"): source})

    def test_domain_citations_must_be_commit_pinned(self):
        sources = reference_sources(self.root, self.tracked)
        citations = validate_domain_references(sources)
        for citation in citations:
            with self.subTest(file=citation["file"], url=citation["url"]):
                path = PurePosixPath(citation["file"])
                changed = dict(sources)
                changed[path] = changed[path].replace(
                    citation["url"],
                    citation["url"].replace(citation["revision"], "main"),
                )
                with self.assertRaisesRegex(IsolationError, "mutable domain citation"):
                    validate_domain_references(changed)

        with self.assertRaisesRegex(IsolationError, "local domain evidence locator"):
            validate_domain_references(
                {
                    **sources,
                    PurePosixPath("README.md"): "see ../azimuth-demo/experiments/multirepo",
                }
            )

    def test_workflow_has_one_checkout_and_one_root_command(self):
        source = (self.root / WORKFLOW).read_text()
        self.assertEqual(validate_workflow(source)["command"], "./scripts/check.sh")
        with self.assertRaisesRegex(IsolationError, "another repository"):
            validate_workflow(source.replace("with:", "with:\n          repository: other/repo", 1))
        with self.assertRaisesRegex(IsolationError, "canonical ./scripts/check.sh"):
            validate_workflow(
                source.replace("./scripts/check.sh", "./experiments/polyglot/check.sh")
            )
        with self.assertRaisesRegex(IsolationError, "retain history"):
            validate_workflow(source.replace("fetch-depth: 0", "fetch-depth: 1"))

    def test_release_qualification_follows_every_experiment_gate(self):
        account = derive_root_account(self.catalog, self.root, self.tracked)
        source = (self.root / ROOT_GATE).read_text()
        execution = validate_root_sequence(source, account)
        self.assertEqual(execution["outcome"], "passed")
        changed = source.replace(
            "./release/check.sh --experiments-executed",
            "./release/check.sh --experiments-executed\n./experiments/polyglot/check.sh",
        ).replace("./experiments/polyglot/check.sh\n", "", 1)
        with self.assertRaisesRegex(IsolationError, "before release qualification"):
            validate_root_sequence(changed, account)

    def test_workflow_receipt_is_exact_revision_evidence(self):
        revision = "a" * 40
        account_fingerprint = "c" * 64
        receipt = {
            "format": "azimuth-github-workflow-receipt",
            "schemaVersion": 1,
            "repository": "drim-dev/azimuth",
            "workflow": ".github/workflows/ci.yml",
            "revision": revision,
            "conclusion": "success",
            "accountFingerprint": account_fingerprint,
            "runUrl": "https://github.com/drim-dev/azimuth/actions/runs/123",
        }
        self.assertEqual(
            validate_workflow_receipt(receipt, revision, account_fingerprint)["revision"],
            revision,
        )
        historical = {**receipt, "revision": "b" * 40}
        self.assertEqual(
            validate_workflow_receipt(
                historical,
                revision,
                account_fingerprint,
                lambda candidate: candidate == historical["revision"],
            )["revision"],
            historical["revision"],
        )
        for field, value in (
            ("revision", "b" * 40),
            ("conclusion", "failure"),
            ("accountFingerprint", "d" * 64),
        ):
            with self.subTest(field=field):
                changed = dict(receipt)
                changed[field] = value
                with self.assertRaisesRegex(IsolationError, f"workflow receipt {field}"):
                    validate_workflow_receipt(changed, revision, account_fingerprint)

    def test_archived_receipt_is_selected_only_for_the_same_account(self):
        revision = "a" * 40
        account_fingerprint = "c" * 64
        receipt = {
            "format": "azimuth-github-workflow-receipt",
            "schemaVersion": 1,
            "repository": "drim-dev/azimuth",
            "workflow": ".github/workflows/ci.yml",
            "revision": revision,
            "conclusion": "success",
            "accountFingerprint": account_fingerprint,
            "runUrl": "https://github.com/drim-dev/azimuth/actions/runs/123",
        }
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            archive = root / "azimuth/changes/archive/2026-08-14-isolation"
            archive.mkdir(parents=True)
            (archive / "workflow-receipt.json").write_text(json.dumps(receipt))
            self.assertEqual(
                archived_workflow_receipt(root, revision, account_fingerprint)["revision"],
                revision,
            )
            self.assertIsNone(
                archived_workflow_receipt(root, revision, "d" * 64)
            )

    def test_current_executable_inputs_have_no_domain_locator(self):
        validate_domain_inputs(
            executable_inputs(self.catalog, self.root, self.tracked)
        )


if __name__ == "__main__":
    unittest.main()
