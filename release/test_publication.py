import argparse
import hashlib
import io
import json
import struct
import tarfile
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from release.orchestrate import assemble, exact_registry_state, expected_subjects, write_account
from release.publication import (
    PublicationError,
    collect_state,
    has_provenance,
    image_manifest,
    package_bytes,
    preflight,
    public_account,
    publish,
    publish_crate,
    publication_workflow_account,
    qualify,
)


REVISION = "a" * 40


class PublicAlphaPublicationTests(unittest.TestCase):
    def setUp(self):
        self.root = Path(__file__).resolve().parent.parent
        self.catalog = json.loads((self.root / "release/artifacts.json").read_text())

    def oci_candidate(self, path, index):
        manifest = json.dumps(
            {"schemaVersion": 2, "manifests": [], "candidate": index},
            separators=(",", ":"),
        ).encode()
        checksum = hashlib.sha256(manifest).hexdigest()
        index_data = json.dumps(
            {
                "schemaVersion": 2,
                "manifests": [
                    {
                        "mediaType": "application/vnd.oci.image.index.v1+json",
                        "digest": f"sha256:{checksum}",
                        "size": len(manifest),
                    }
                ],
            }
        ).encode()
        with tarfile.open(path, "w") as archive:
            for name, content in (
                ("index.json", index_data),
                (f"blobs/sha256/{checksum}", manifest),
            ):
                member = tarfile.TarInfo(name)
                member.size = len(content)
                archive.addfile(member, io.BytesIO(content))

    def retained_account(self, directory):
        candidates = directory / "candidates"
        candidates.mkdir()
        for index, subject in enumerate(expected_subjects(self.catalog)):
            path = candidates / subject["filename"]
            if subject["kind"] == "image":
                self.oci_candidate(path, index)
            else:
                path.write_bytes(f"candidate-{index}".encode())
        account = assemble(candidates, REVISION, self.catalog["release"]["tag"], self.root)
        account_path = directory / "candidates.json"
        write_account(account, account_path)
        return account, candidates, account_path, directory / "SHA256SUMS"

    def test_package_adapters_distinguish_absence_from_exact_bytes(self):
        subjects = {
            item["ecosystem"]: item
            for item in expected_subjects(self.catalog)
            if item["kind"] == "package"
        }
        with patch("release.publication.request", return_value=(404, {}, b"missing")):
            for ecosystem in ("cargo", "nuget"):
                with self.subTest(ecosystem=ecosystem):
                    content, _ = package_bytes(subjects[ecosystem], "0.1.0-alpha.1")
                    self.assertIsNone(content)

        cargo_index = json.dumps({"name": "azimuth", "vers": "0.1.0-alpha.1"}).encode()
        with patch(
            "release.publication.request",
            side_effect=[(200, {}, cargo_index), (200, {}, b"retained-crate-bytes")],
        ):
            content, url = package_bytes(subjects["cargo"], "0.1.0-alpha.1")
        self.assertEqual(content, b"retained-crate-bytes")
        self.assertEqual(
            url,
            "https://static.crates.io/crates/azimuth/azimuth-0.1.0-alpha.1.crate",
        )

        npm_metadata = {
            "versions": {
                "0.1.0-alpha.1": {"dist": {"tarball": "https://registry.test/package.tgz"}}
            }
        }
        responses = [
            (200, {}, json.dumps(npm_metadata).encode()),
            (200, {}, b"retained-npm-bytes"),
        ]
        with patch("release.publication.request", side_effect=responses):
            content, url = package_bytes(subjects["npm"], "0.1.0-alpha.1")
        self.assertEqual(content, b"retained-npm-bytes")
        self.assertEqual(url, "https://registry.test/package.tgz")

    def test_public_account_binds_each_image_index_digest_to_its_retained_archive(self):
        with tempfile.TemporaryDirectory() as temporary:
            retained, candidates, _, _ = self.retained_account(Path(temporary))
            account = public_account(retained, candidates)
            images = [subject for subject in account["subjects"] if subject["kind"] == "image"]
            self.assertEqual(len(images), 2)
            self.assertTrue(all(subject["registryDigest"].startswith("sha256:") for subject in images))
            self.assertTrue(all(subject["retainedSha256"] != subject["sha256"] for subject in images))

            selected = images[0]
            archive = candidates / selected["filename"]
            with tarfile.open(archive, "a") as package:
                changed = b"changed"
                member = tarfile.TarInfo(
                    f"blobs/sha256/{selected['registryDigest'].removeprefix('sha256:')}"
                )
                member.size = len(changed)
                package.addfile(member, io.BytesIO(changed))
            with self.assertRaisesRegex(PublicationError, "checksum differs"):
                public_account(retained, candidates)

    def test_provider_errors_are_unknown_and_fail_closed(self):
        subject = next(
            item
            for item in expected_subjects(self.catalog)
            if item.get("ecosystem") == "nuget"
        )
        for status in (401, 403, 429, 500):
            with self.subTest(status=status), patch(
                "release.publication.request", return_value=(status, {}, b"failure")
            ):
                with self.assertRaisesRegex(PublicationError, f"HTTP {status}"):
                    package_bytes(subject, "0.1.0-alpha.1")

    def test_public_state_covers_every_exact_registry_target(self):
        with tempfile.TemporaryDirectory() as temporary:
            retained, candidates, account_path, sums_path = self.retained_account(Path(temporary))
            account = public_account(retained, candidates)
            by_name = {subject["filename"]: subject for subject in account["subjects"]}
            support = {
                "candidates.json": account_path.read_bytes(),
                "SHA256SUMS": sums_path.read_bytes(),
            }
            assets = [
                {"name": name, "url": name, "browser_download_url": f"https://example/{name}"}
                for name in support
            ]
            assets.extend(
                {
                    "name": subject["filename"],
                    "url": subject["filename"],
                    "browser_download_url": f"https://example/{subject['filename']}",
                }
                for subject in account["subjects"]
                if subject["kind"] == "native"
            )

            def package_result(subject, _version):
                return (candidates / subject["filename"]).read_bytes(), "https://example/package"

            def asset_result(asset):
                if asset["name"] in support:
                    return support[asset["name"]]
                return (candidates / by_name[asset["name"]]["filename"]).read_bytes()

            image_values = iter(
                (
                    subject["registryDigest"].removeprefix("sha256:"),
                    subject["platforms"],
                )
                for subject in account["subjects"]
                if subject["kind"] == "image"
            )
            with patch("release.publication.package_bytes", side_effect=package_result), patch(
                "release.publication.github_release",
                return_value={
                    "tag_name": account["tag"],
                    "prerelease": True,
                    "assets": assets,
                },
            ), patch("release.publication.github_asset_bytes", side_effect=asset_result), patch(
                "release.publication.image_manifest", side_effect=lambda *_: next(image_values)
            ), patch("release.publication.has_provenance", return_value=True):
                state = collect_state(account, account_path, sums_path)
            self.assertEqual(state["missingReleaseAssets"], [])
            self.assertEqual(len(state["targets"]), 10)
            self.assertEqual(
                {key: value["sha256"] for key, value in state["targets"].items()},
                {
                    key: value["sha256"]
                    for key, value in exact_registry_state(account)["targets"].items()
                },
            )

    def test_conflicting_release_support_asset_fails_before_planning(self):
        with tempfile.TemporaryDirectory() as temporary:
            retained, candidates, account_path, sums_path = self.retained_account(Path(temporary))
            account = public_account(retained, candidates)
            with patch(
                "release.publication.github_release",
                return_value={
                    "tag_name": account["tag"],
                    "prerelease": True,
                    "assets": [{"name": "candidates.json", "url": "candidate"}],
                },
            ), patch("release.publication.github_asset_bytes", return_value=b"different"):
                with self.assertRaisesRegex(PublicationError, "support asset"):
                    collect_state(account, account_path, sums_path)

    def test_preflight_records_zero_writes_and_credentials_gate_publication(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            retained, candidates, account_path, sums_path = self.retained_account(directory)
            account = public_account(retained, candidates)
            exact = exact_registry_state(account)
            arguments = argparse.Namespace(
                account=account_path,
                candidates=candidates,
                sums=sums_path,
                root=self.root,
                repository="drim-dev/azimuth",
                tag=account["tag"],
                run_revision=REVISION,
                rehearsal_run="123",
                state_out=directory / "state.json",
                plan_out=directory / "plan.json",
                receipt_out=directory / "preflight.json",
                require_credentials=False,
            )
            with patch("release.publication.verify_tag"), patch(
                "release.publication.annotated_tag_revision", return_value=REVISION
            ), patch("release.publication.collect_state", return_value=exact), patch(
                "release.publication.credential_account", return_value={"ready": False}
            ):
                preflight(arguments)
            receipt = json.loads(arguments.receipt_out.read_text())
            self.assertEqual(receipt["writes"], 0)
            self.assertEqual(receipt["plan"]["publish"], [])

            arguments.require_credentials = True
            with patch("release.publication.verify_tag"), patch(
                "release.publication.annotated_tag_revision", return_value=REVISION
            ), patch("release.publication.collect_state", return_value=exact), patch(
                "release.publication.credential_account", return_value={"ready": False}
            ):
                with self.assertRaisesRegex(PublicationError, "not ready"):
                    preflight(arguments)

    def test_publish_consumes_only_the_planner_selected_absent_target(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            retained, candidates, account_path, sums_path = self.retained_account(directory)
            account = public_account(retained, candidates)
            state = exact_registry_state(account)
            selected = account["subjects"][0]["key"]
            del state["targets"][selected]
            state.update({"releaseExists": True, "missingReleaseAssets": []})
            state_path = directory / "state.json"
            plan_path = directory / "plan.json"
            state_path.write_text(json.dumps(state))
            plan_path.write_text(
                json.dumps({"publish": [selected], "preserve": sorted(state["targets"])})
            )
            arguments = argparse.Namespace(
                account=account_path,
                candidates=candidates,
                sums=sums_path,
                root=self.root,
                repository="drim-dev/azimuth",
                state=state_path,
                plan=plan_path,
                out=directory / "result.json",
            )
            with patch(
                "release.publication.credential_account", return_value={"ready": True}
            ), patch("release.publication.publish_target") as publish_one:
                publish(arguments)
            self.assertEqual(publish_one.call_args.args[0]["key"], selected)
            self.assertEqual(json.loads(arguments.out.read_text())["published"], [selected])

    def test_crates_upload_body_contains_the_exact_retained_archive(self):
        with tempfile.TemporaryDirectory() as temporary:
            archive = Path(temporary) / "azimuth-0.1.0-alpha.1.crate"
            manifest = (
                "[package]\nname='azimuth'\nversion='0.1.0-alpha.1'\n"
                "description='test'\nlicense='Apache-2.0'\nreadme='README.md'\n"
            ).encode()
            with tarfile.open(archive, "w:gz") as package:
                for name, content in (
                    ("azimuth-0.1.0-alpha.1/Cargo.toml", manifest),
                    ("azimuth-0.1.0-alpha.1/README.md", b"readme"),
                ):
                    member = tarfile.TarInfo(name)
                    member.size = len(content)
                    package.addfile(member, io.BytesIO(content))
            captured = {}

            def upload(_url, **values):
                captured.update(values)
                return 200, {}, b"{}"

            with patch.dict("os.environ", {"CARGO_REGISTRY_TOKEN": "secret"}), patch(
                "release.publication.request", side_effect=upload
            ):
                publish_crate(archive)
            body = captured["body"]
            metadata_length = struct.unpack("<I", body[:4])[0]
            archive_offset = 4 + metadata_length
            archive_length = struct.unpack("<I", body[archive_offset:archive_offset + 4])[0]
            retained = body[archive_offset + 4:]
            self.assertEqual(archive_length, archive.stat().st_size)
            self.assertEqual(retained, archive.read_bytes())
            self.assertEqual(captured["method"], "PUT")
            self.assertEqual(captured["headers"]["Accept"], "application/json")

    def test_image_state_filters_attestation_descriptors_from_platforms(self):
        manifest = {
            "schemaVersion": 2,
            "manifests": [
                {"platform": {"os": "linux", "architecture": "arm64"}},
                {"platform": {"os": "unknown", "architecture": "unknown"}},
                {"platform": {"os": "linux", "architecture": "amd64"}},
            ],
        }
        raw = json.dumps(manifest, separators=(",", ":")).encode()
        subject = {"key": "image:api", "identity": "ghcr.io/drim-dev/api"}
        result = type("Result", (), {"returncode": 0, "stdout": raw, "stderr": b""})()
        with patch("release.publication.run", return_value=result):
            checksum, platforms = image_manifest(subject, "0.1.0-alpha.1")
        self.assertEqual(checksum, hashlib.sha256(raw).hexdigest())
        self.assertEqual(platforms, ["linux/amd64", "linux/arm64"])

    def test_provenance_requires_both_digest_and_tagged_revision(self):
        def payload(checksum, revision):
            return {
                "subject": [{"digest": {"sha256": checksum}}],
                "predicate": {
                    "buildDefinition": {
                        "resolvedDependencies": [{"digest": {"gitCommit": revision}}]
                    }
                },
            }

        checksum = "b" * 64
        with patch(
            "release.publication.attestation_payloads",
            return_value=[payload(checksum, REVISION)],
        ):
            self.assertTrue(has_provenance(checksum, REVISION))
            self.assertFalse(has_provenance(checksum, "c" * 40))

    def test_publication_workflow_is_owner_dispatched_and_never_rebuilds(self):
        account = publication_workflow_account(self.root)
        self.assertEqual(account["trigger"], "workflow_dispatch")
        self.assertEqual(account["candidateBuilds"], 0)

    def test_publication_workflow_fails_static_credential_and_provenance_mutations(self):
        source = (self.root / ".github/workflows/publish.yml").read_text()
        mutations = {
            "credential": source.replace("credential_args+=(--require-credentials)", "true"),
            "provenance": source.replace("push-to-registry: true", "push-to-registry: false"),
            "rebuild": source.replace(
                "python3 release/publication.py publish",
                "python3 release/candidates.py packages && python3 release/publication.py publish",
            ),
        }
        for name, changed in mutations.items():
            with self.subTest(name=name), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                workflow = root / ".github/workflows"
                workflow.mkdir(parents=True)
                (workflow / "publish.yml").write_text(changed)
                with self.assertRaises(PublicationError):
                    publication_workflow_account(root)

    def test_qualification_declares_realization_without_fabricating_operational_evidence(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary)
            qualify(argparse.Namespace(root=self.root, out=output))
            linkage = json.loads((output / "publication-linkage.json").read_text())
            qualification = json.loads((output / "publication.json").read_text())
            self.assertEqual(len(linkage["realizes"]), 7)
            self.assertEqual(linkage["covers"], [])
            self.assertEqual(qualification["operationalEvidence"], "pending")


if __name__ == "__main__":
    unittest.main()
