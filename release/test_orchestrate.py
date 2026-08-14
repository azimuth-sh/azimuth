import copy
import json
import subprocess
import tempfile
import unittest
from pathlib import Path

from release.orchestrate import (
    OrchestrationError,
    assemble,
    expected_subjects,
    image_matrix,
    native_matrix,
    plan_publication,
    rehearse_publication,
    digest,
    expected_release_jobs,
    validate_completion,
    validate_ordinary_receipt,
    validate_release_receipt,
    verify,
    verify_tag,
    workflow_account,
)


REVISION = "a" * 40


class ReleaseOrchestrationTests(unittest.TestCase):
    def setUp(self):
        self.root = Path(__file__).resolve().parent.parent
        self.catalog = json.loads((self.root / "release/artifacts.json").read_text())

    def candidates(self, root):
        for index, subject in enumerate(expected_subjects(self.catalog)):
            (root / subject["filename"]).write_bytes(f"candidate-{index}".encode())

    def account(self, root):
        self.candidates(root)
        return assemble(root, REVISION, self.catalog["release"]["tag"], self.root)

    def exact_state(self, account, provenance=True):
        return {
            "targets": {
                subject["key"]: {
                    "identity": subject["identity"],
                    "sha256": subject["sha256"],
                    "provenance": provenance,
                    **(
                        {"platforms": subject["platforms"]}
                        if "platforms" in subject
                        else {}
                    ),
                }
                for subject in account["subjects"]
            }
        }

    def ordinary_receipt(self):
        return {
            "format": "azimuth-ordinary-workflow-receipt",
            "schemaVersion": 1,
            "workflow": ".github/workflows/ci.yml",
            "sourceRevision": REVISION,
            "executionRevision": "b" * 40,
            "runUrl": "https://github.com/drim-dev/azimuth/actions/runs/123",
            "conclusion": "success",
            "durationSeconds": 120,
            "workflowSha256": digest(self.root / ".github/workflows/ci.yml"),
            "rootGateSha256": digest(self.root / "scripts/check.sh"),
        }

    def release_receipt(self):
        filenames = [item["filename"] for item in expected_subjects(self.catalog)]
        return {
            "format": "azimuth-release-workflow-receipt",
            "schemaVersion": 1,
            "workflow": ".github/workflows/release.yml",
            "sourceRevision": REVISION,
            "executionRevision": "b" * 40,
            "runUrl": "https://github.com/drim-dev/azimuth/actions/runs/123",
            "conclusion": "success",
            "workflowSha256": digest(self.root / ".github/workflows/release.yml"),
            "accountSha256": digest(self.root / "release/orchestrate.py"),
            "consumerSha256": digest(self.root / "release/candidates.py"),
            "jobs": expected_release_jobs(self.catalog),
            "subjects": filenames,
            "attestedSubjects": filenames,
            "candidateAccountSha256": "c" * 64,
        }

    def test_selected_matrices_derive_from_the_catalog(self):
        natives = native_matrix(self.catalog)
        images = image_matrix(self.catalog)
        self.assertEqual(
            [item["target"] for item in natives],
            self.catalog["nativeBinaries"]["targets"],
        )
        self.assertEqual([item["id"] for item in images], ["assurance-api", "assurance-web"])
        self.assertTrue(all(item["platforms"] == "linux/amd64,linux/arm64" for item in images))

    def test_hosted_receipts_bind_timing_jobs_subjects_and_executable_inputs(self):
        ancestor = lambda _root, revision: revision == REVISION
        ordinary = self.ordinary_receipt()
        release = self.release_receipt()
        self.assertEqual(
            validate_ordinary_receipt(ordinary, self.root, ancestor),
            ordinary,
        )
        self.assertEqual(validate_release_receipt(release, self.root, ancestor), release)
        mutations = {
            "ordinary duration": (ordinary, "durationSeconds", 45 * 60),
            "ordinary workflow": (ordinary, "workflowSha256", "f" * 64),
            "release jobs": (release, "jobs", release["jobs"][:-1]),
            "release subjects": (release, "subjects", release["subjects"][:-1]),
            "release provenance": (
                release,
                "attestedSubjects",
                release["attestedSubjects"][:-1],
            ),
            "release consumer": (release, "consumerSha256", "f" * 64),
        }
        for name, (receipt, field, value) in mutations.items():
            with self.subTest(name=name):
                changed = copy.deepcopy(receipt)
                changed[field] = value
                with self.assertRaises(OrchestrationError):
                    if receipt is ordinary:
                        validate_ordinary_receipt(changed, self.root, ancestor)
                    else:
                        validate_release_receipt(changed, self.root, ancestor)

    def test_exact_candidate_population_assembles_and_verifies(self):
        with tempfile.TemporaryDirectory() as temporary:
            candidate_root = Path(temporary)
            account = self.account(candidate_root)
            self.assertEqual(len(account["subjects"]), 10)
            self.assertEqual(verify(account, candidate_root, self.root), account)

    def test_each_missing_and_duplicate_candidate_and_an_unexpected_candidate_fail(self):
        for subject in expected_subjects(self.catalog):
            for mutation in ("missing", "duplicate"):
                with self.subTest(subject=subject["key"], mutation=mutation), \
                        tempfile.TemporaryDirectory() as temporary:
                    candidate_root = Path(temporary)
                    self.candidates(candidate_root)
                    filename = subject["filename"]
                    if mutation == "missing":
                        (candidate_root / filename).unlink()
                    else:
                        duplicate = candidate_root / "nested"
                        duplicate.mkdir()
                        (duplicate / filename).write_bytes(b"duplicate")
                    with self.assertRaises(OrchestrationError):
                        assemble(
                            candidate_root,
                            REVISION,
                            self.catalog["release"]["tag"],
                            self.root,
                        )
        with tempfile.TemporaryDirectory() as temporary:
            candidate_root = Path(temporary)
            self.candidates(candidate_root)
            (candidate_root / "unexpected.bin").write_bytes(b"unexpected")
            with self.assertRaisesRegex(OrchestrationError, "unexpected"):
                assemble(candidate_root, REVISION, self.catalog["release"]["tag"], self.root)

    def test_tag_revision_and_each_candidate_byte_set_fail_independently(self):
        with tempfile.TemporaryDirectory() as temporary:
            candidate_root = Path(temporary)
            account = self.account(candidate_root)
            with self.assertRaisesRegex(OrchestrationError, "tag"):
                assemble(candidate_root, REVISION, "v0.1.0-alpha.2", self.root)
            with self.assertRaisesRegex(OrchestrationError, "revision"):
                assemble(candidate_root, "short", self.catalog["release"]["tag"], self.root)
            for subject in account["subjects"]:
                with self.subTest(subject=subject["key"]):
                    path = candidate_root / subject["filename"]
                    original = path.read_bytes()
                    path.write_bytes(original + b"changed")
                    with self.assertRaisesRegex(OrchestrationError, "byte size|checksum"):
                        verify(account, candidate_root, self.root)
                    path.write_bytes(original)

    def test_annotated_tag_must_name_the_candidate_revision(self):
        with tempfile.TemporaryDirectory() as temporary:
            repository = Path(temporary)
            subprocess.run(["git", "init", "--quiet"], cwd=repository, check=True)
            (repository / "subject").write_text("candidate")
            subprocess.run(["git", "add", "subject"], cwd=repository, check=True)
            subprocess.run(
                [
                    "git",
                    "-c",
                    "user.name=Azimuth",
                    "-c",
                    "user.email=azimuth@example.test",
                    "commit",
                    "--quiet",
                    "--message",
                    "candidate",
                ],
                cwd=repository,
                check=True,
            )
            revision = subprocess.run(
                ["git", "rev-parse", "HEAD"],
                cwd=repository,
                check=True,
                capture_output=True,
                text=True,
            ).stdout.strip()
            subprocess.run(
                ["git", "tag", "--annotate", "candidate", "--message", "candidate"],
                cwd=repository,
                check=True,
            )
            verify_tag(repository, "candidate", revision)
            with self.assertRaisesRegex(OrchestrationError, "does not name revision"):
                verify_tag(repository, "candidate", "f" * 40)

    def test_each_absent_target_is_the_only_selected_publication(self):
        with tempfile.TemporaryDirectory() as temporary:
            account = self.account(Path(temporary))
            exact = self.exact_state(account)
            for subject in account["subjects"]:
                with self.subTest(subject=subject["key"]):
                    state = copy.deepcopy(exact)
                    del state["targets"][subject["key"]]
                    plan = plan_publication(account, state)
                    self.assertEqual(plan["publish"], [subject["key"]])
                    self.assertNotIn(subject["key"], plan["preserve"])

    def test_exact_targets_are_preserved_and_conflicts_fail_closed(self):
        with tempfile.TemporaryDirectory() as temporary:
            account = self.account(Path(temporary))
            exact = self.exact_state(account)
            plan = plan_publication(account, exact)
            self.assertEqual(plan["publish"], [])
            self.assertEqual(len(plan["preserve"]), 10)
            for kind, field in (
                ("package", "sha256"),
                ("native", "identity"),
                ("image", "platforms"),
            ):
                with self.subTest(kind=kind, field=field):
                    state = copy.deepcopy(exact)
                    subject = next(item for item in account["subjects"] if item["kind"] == kind)
                    state["targets"][subject["key"]][field] = (
                        ["linux/s390x"] if field == "platforms" else "different"
                    )
                    with self.assertRaisesRegex(OrchestrationError, "conflicts"):
                        plan_publication(account, state)

    def test_completion_needs_every_target_and_provenance(self):
        with tempfile.TemporaryDirectory() as temporary:
            account = self.account(Path(temporary))
            exact = self.exact_state(account)
            self.assertEqual(validate_completion(account, exact)["outcome"], "complete")
            for subject in account["subjects"]:
                with self.subTest(subject=subject["key"], condition="missing"):
                    missing = copy.deepcopy(exact)
                    missing["targets"].pop(subject["key"])
                    with self.assertRaisesRegex(OrchestrationError, "omits"):
                        validate_completion(account, missing)
                with self.subTest(subject=subject["key"], condition="provenance"):
                    unproven = copy.deepcopy(exact)
                    unproven["targets"][subject["key"]]["provenance"] = False
                    with self.assertRaisesRegex(OrchestrationError, "provenance"):
                        validate_completion(account, unproven)
                if subject["kind"] == "image":
                    with self.subTest(subject=subject["key"], condition="platforms"):
                        changed = copy.deepcopy(exact)
                        changed["targets"][subject["key"]]["platforms"] = ["linux/s390x"]
                        with self.assertRaisesRegex(OrchestrationError, "conflicts"):
                            validate_completion(account, changed)

    def test_publication_rehearsal_ranges_over_every_target_and_registry_kind(self):
        with tempfile.TemporaryDirectory() as temporary:
            account = self.account(Path(temporary))
            result = rehearse_publication(account)
            self.assertEqual(len(result["preserved"]), 10)
            self.assertEqual(len(result["individuallyAbsent"]), 10)
            self.assertEqual(len(result["rejectedConflictKinds"]), 3)
            self.assertEqual(result["completion"], "passed")

    def test_workflows_separate_ordinary_and_release_image_accounts(self):
        account = workflow_account(self.root)
        self.assertFalse(account["releaseImagesInOrdinaryGate"])
        self.assertEqual(account["releaseLanes"], ["packages", "native", "images", "account"])

    def test_each_release_lane_is_required_by_the_workflow_account(self):
        release = (self.root / ".github/workflows/release.yml").read_text()
        ordinary = (self.root / ".github/workflows/ci.yml").read_text()
        gate = (self.root / "scripts/check.sh").read_text()
        for lane in ("packages", "native", "images", "account"):
            with self.subTest(lane=lane), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                (root / ".github/workflows").mkdir(parents=True)
                (root / "scripts").mkdir()
                (root / ".github/workflows/ci.yml").write_text(ordinary)
                (root / "scripts/check.sh").write_text(gate)
                changed = release.replace(f"  {lane}:\n", "", 1)
                (root / ".github/workflows/release.yml").write_text(changed)
                with self.assertRaisesRegex(OrchestrationError, f"lane {lane!r}"):
                    workflow_account(root)


if __name__ == "__main__":
    unittest.main()
