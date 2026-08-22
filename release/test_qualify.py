import copy
import json
import re
import tempfile
import unittest
from pathlib import Path

from release.acceptance import APPROVED_CONTRACT
from release.qualify import (
    QualificationError,
    approved_contract_differences,
    catalog_at,
    stable_content_account,
    validate_approved_contract,
    validate_catalog,
    validate_file_set,
    validate_source_metadata,
    write_linkage,
)


ROOT = Path(__file__).resolve().parent.parent


class ReleaseQualificationTests(unittest.TestCase):
    def setUp(self):
        self.catalog = catalog_at(ROOT)

    def test_catalog_matches_approved_contract(self):
        self.assertEqual(approved_contract_differences(self.catalog, APPROVED_CONTRACT), [])
        validate_catalog(self.catalog, ROOT)
        validate_source_metadata(self.catalog, ROOT)

    def test_linkage_uses_only_v2_collections_and_fingerprints(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary)
            write_linkage(ROOT, output)
            linkage = json.loads((output / "linkage.json").read_text())
        self.assertEqual(
            set(linkage),
            {
                "realizes",
                "check_implementations",
                "mechanism_implementations",
                "class_members",
                "enumerations",
                "artifacts",
            },
        )
        self.assertTrue(
            all(
                re.fullmatch(r"sha256:[0-9a-f]{64}", entry["source_fingerprint"])
                for entry in linkage["realizes"]
            )
        )

    def test_each_contract_dimension_drifts_independently(self):
        mutations = {
            # Drift values are deliberately unreleasable so a future version bump cannot
            # make the mutation a no-op and silently disarm this test.
            "version": lambda item: item["release"].__setitem__("version", "0.0.0-drift"),
            "tag": lambda item: item["release"].__setitem__("tag", "v0.0.0-drift"),
            "license": lambda item: item["release"].__setitem__("license", "MIT"),
            "repository": lambda item: item["release"].__setitem__(
                "repository", "https://example.invalid/source"
            ),
            "homepage": lambda item: item["release"].__setitem__(
                "homepage", "https://example.invalid"
            ),
            "identities": lambda item: item["packages"][0].__setitem__("identity", "other"),
            "nativeTargets": lambda item: item["nativeBinaries"]["targets"].pop(),
            "imagePlatforms": lambda item: item["images"][0]["platforms"].pop(),
            "supportedSurfaces": lambda item: item["supportedSurfaces"].pop(),
            "experimentalSource": lambda item: item["experimentalSource"].pop(),
        }
        for dimension, mutate in mutations.items():
            with self.subTest(dimension=dimension):
                changed = copy.deepcopy(self.catalog)
                mutate(changed)
                self.assertEqual(
                    approved_contract_differences(changed, APPROVED_CONTRACT),
                    [dimension],
                )
                with self.assertRaisesRegex(QualificationError, dimension):
                    validate_approved_contract(changed)

    def test_set_reordering_does_not_change_the_contract(self):
        changed = copy.deepcopy(self.catalog)
        changed["packages"].reverse()
        changed["images"].reverse()
        changed["nativeBinaries"]["targets"].reverse()
        changed["supportedSurfaces"].reverse()
        changed["experimentalSource"].reverse()
        for image in changed["images"]:
            image["platforms"].reverse()
        self.assertEqual(approved_contract_differences(changed, APPROVED_CONTRACT), [])

    def test_public_package_cannot_live_under_experimental_source(self):
        changed = copy.deepcopy(self.catalog)
        changed["packages"][0]["manifest"] = "experiments/polyglot/model/spec.md"
        with self.assertRaisesRegex(QualificationError, "public package is inside experimental"):
            validate_catalog(changed, ROOT)

    def test_packed_contents_fail_for_missing_and_unexpected_files(self):
        package = {
            "id": "fixture",
            "requiredFiles": ["package/index.js"],
            "allowedFiles": ["package/index.js"],
        }
        with self.assertRaisesRegex(QualificationError, "omits required file"):
            validate_file_set(package, [])
        with self.assertRaisesRegex(QualificationError, "undeclared files"):
            validate_file_set(package, ["package/index.js", "package/secret.env"])

    def test_generated_archive_names_have_a_stable_content_account(self):
        package = {
            "allowedFiles": ["package/services/metadata/*.psmdcp", "lib/product.dll"],
        }
        first = ["package/services/metadata/first.psmdcp", "lib/product.dll"]
        rebuilt = ["package/services/metadata/second.psmdcp", "lib/product.dll"]
        self.assertEqual(
            stable_content_account(package, first),
            stable_content_account(package, rebuilt),
        )

    def test_catalog_rejects_duplicate_public_identity(self):
        changed = copy.deepcopy(self.catalog)
        changed["packages"][1]["identity"] = changed["packages"][0]["identity"]
        with self.assertRaisesRegex(QualificationError, "public identities are not unique"):
            validate_catalog(changed, ROOT)


if __name__ == "__main__":
    unittest.main()
