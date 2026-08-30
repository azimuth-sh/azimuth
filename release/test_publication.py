import argparse
import hashlib
import io
import json
import re
import struct
import tarfile
import tempfile
import unittest
import zipfile
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

from release.orchestrate import expected_subjects, release_artifacts
from release.publication import (
    PublicationError,
    annotated_tag_revision,
    collect_state,
    credential_account,
    github_release,
    has_provenance,
    image_manifest,
    image_state,
    normalize_npm_dist_tags,
    nuget_payload_digest,
    package_bytes,
    preflight,
    public_release,
    publish,
    publish_crate,
    publish_target,
    publication_workflow_structure,
    qualify,
    registry_publication_plan,
    subject_provenance,
    validate_registry_completion,
    write_checksums,
    verify_nuget_repository_signature,
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

    def retained_release(self, directory):
        candidates = directory / "candidates"
        candidates.mkdir()
        for index, subject in enumerate(expected_subjects(self.catalog)):
            path = candidates / subject["filename"]
            if subject["kind"] == "image":
                self.oci_candidate(path, index)
            elif subject.get("ecosystem") == "nuget":
                with zipfile.ZipFile(path, "w") as package:
                    package.writestr(f"payload-{index}.txt", f"candidate-{index}")
            else:
                path.write_bytes(f"candidate-{index}".encode())
        release = release_artifacts(
            candidates, REVISION, self.catalog["release"]["tag"], self.root
        )
        sums_path = directory / "SHA256SUMS"
        write_checksums(release, sums_path)
        return release, candidates, sums_path

    def exact_registry_state(self, release):
        return {
            "targets": {
                subject["key"]: {
                    "identity": subject["identity"],
                    "sha256": subject["sha256"],
                    "provenance": True,
                    **(
                        {"platforms": subject["platforms"]}
                        if "platforms" in subject
                        else {}
                    ),
                }
                for subject in release["subjects"]
            }
        }

    def exact_public_state(self, account):
        state = self.exact_registry_state(account)
        for subject in account["subjects"]:
            if subject.get("ecosystem") == "npm":
                # With no stable version, an exact state also has `latest` on this prerelease:
                # npm requires `latest` to resolve and there is no better target.
                state["targets"][subject["key"]]["distTags"] = {
                    "alpha": account["version"],
                    "latest": account["version"],
                }
                state["targets"][subject["key"]]["stableVersions"] = []
        return state

    def test_package_adapters_distinguish_absence_from_exact_bytes(self):
        subjects = {
            item["ecosystem"]: item
            for item in expected_subjects(self.catalog)
            if item["kind"] == "package"
        }
        with patch("release.publication.request", return_value=(404, {}, b"missing")):
            for ecosystem in ("cargo", "nuget"):
                with self.subTest(ecosystem=ecosystem):
                    content, _ = package_bytes(subjects[ecosystem], "0.1.0-alpha.2")
                    self.assertIsNone(content)

        cargo_index = json.dumps({"name": "azimuth", "vers": "0.1.0-alpha.2"}).encode()
        with patch(
            "release.publication.request",
            side_effect=[(200, {}, cargo_index), (200, {}, b"retained-crate-bytes")],
        ):
            content, url = package_bytes(subjects["cargo"], "0.1.0-alpha.2")
        self.assertEqual(content, b"retained-crate-bytes")
        self.assertEqual(
            url,
            "https://static.crates.io/crates/azimuth/azimuth-0.1.0-alpha.2.crate",
        )

        npm_metadata = {
            "versions": {
                "0.1.0-alpha.2": {
                    "dist": {"tarball": "https://registry.npmjs.org/package.tgz"}
                }
            }
        }
        responses = [
            (200, {}, json.dumps(npm_metadata).encode()),
            (200, {}, b"retained-npm-bytes"),
        ]
        with patch("release.publication.request", side_effect=responses):
            content, url = package_bytes(subjects["npm"], "0.1.0-alpha.2")
        self.assertEqual(content, b"retained-npm-bytes")
        self.assertEqual(url, "https://registry.npmjs.org/package.tgz")

    def test_npm_tarball_must_remain_on_the_registry_https_origin(self):
        subject = next(
            item
            for item in expected_subjects(self.catalog)
            if item.get("ecosystem") == "npm"
        )
        for url in ("file:///tmp/package.tgz", "https://registry.example/package.tgz"):
            metadata = {
                "versions": {"0.1.0-alpha.2": {"dist": {"tarball": url}}}
            }
            with self.subTest(url=url), patch(
                "release.publication.request",
                return_value=(200, {}, json.dumps(metadata).encode()),
            ) as registry_request:
                with self.assertRaisesRegex(PublicationError, "npm registry HTTPS URL"):
                    package_bytes(subject, "0.1.0-alpha.2")
                registry_request.assert_called_once()

    def test_tag_lookup_uses_the_requested_checkout(self):
        root = Path("/requested/checkout")
        results = [
            SimpleNamespace(stdout=b"tag\n"),
            SimpleNamespace(stdout=f"{REVISION}\n".encode()),
        ]
        with patch("release.publication.run", side_effect=results) as git:
            self.assertEqual(annotated_tag_revision(root, "v0.1.0-alpha.2"), REVISION)
        self.assertEqual([call.kwargs["cwd"] for call in git.call_args_list], [root, root])

    def test_public_release_binds_each_image_index_digest_to_its_retained_archive(self):
        with tempfile.TemporaryDirectory() as temporary:
            retained, candidates, _ = self.retained_release(Path(temporary))
            account = public_release(retained, candidates)
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
                public_release(retained, candidates)

    def test_nuget_repository_signature_preserves_payload_identity(self):
        retained = io.BytesIO()
        with zipfile.ZipFile(retained, "w") as package:
            package.writestr("package.nuspec", b"metadata")
            package.writestr("lib/package.dll", b"assembly")
        published = io.BytesIO()
        with zipfile.ZipFile(published, "w") as package:
            package.writestr("lib/package.dll", b"assembly")
            package.writestr("package.nuspec", b"metadata")
            package.writestr(".signature.p7s", b"repository-signature")

        self.assertNotEqual(
            hashlib.sha256(retained.getvalue()).digest(),
            hashlib.sha256(published.getvalue()).digest(),
        )
        self.assertEqual(
            nuget_payload_digest(retained.getvalue()),
            nuget_payload_digest(published.getvalue()),
        )
        changed = io.BytesIO()
        with zipfile.ZipFile(changed, "w") as package:
            package.writestr("package.nuspec", b"changed")
            package.writestr("lib/package.dll", b"assembly")
            package.writestr(".signature.p7s", b"repository-signature")
        self.assertNotEqual(
            nuget_payload_digest(retained.getvalue()),
            nuget_payload_digest(changed.getvalue()),
        )

        verified = SimpleNamespace(
            returncode=0,
            stdout=b"Signature type: Repository\n",
            stderr=b"",
        )
        with patch("release.publication.run", return_value=verified):
            verify_nuget_repository_signature(published.getvalue())
        rejected = SimpleNamespace(returncode=1, stdout=b"", stderr=b"invalid")
        with patch("release.publication.run", return_value=rejected), self.assertRaisesRegex(
            PublicationError, "signature verification failed"
        ):
            verify_nuget_repository_signature(published.getvalue())

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
                    package_bytes(subject, "0.1.0-alpha.2")

    def test_malformed_provider_state_and_image_command_failures_are_unknown(self):
        subjects = {
            item.get("ecosystem"): item
            for item in expected_subjects(self.catalog)
            if item["kind"] == "package"
        }
        for ecosystem in ("cargo", "npm"):
            with self.subTest(ecosystem=ecosystem), patch(
                "release.publication.request", return_value=(200, {}, b"{")
            ):
                with self.assertRaisesRegex(PublicationError, "malformed"):
                    package_bytes(subjects[ecosystem], "0.1.0-alpha.2")

        image = {"key": "image:api", "identity": "ghcr.io/example/api"}
        malformed = SimpleNamespace(returncode=0, stdout=b"{", stderr=b"")
        with patch("release.publication.run", return_value=malformed):
            with self.assertRaisesRegex(PublicationError, "manifest is malformed"):
                image_manifest(image, "0.1.0-alpha.2")
        failed = SimpleNamespace(returncode=1, stdout=b"", stderr=b"unauthorized")
        with patch("release.publication.run", return_value=failed):
            with self.assertRaisesRegex(PublicationError, "image state failed"):
                image_manifest(image, "0.1.0-alpha.2")

        malformed = SimpleNamespace(returncode=0, stdout=b"{", stderr=b"")
        with patch("release.publication.run", return_value=malformed):
            with self.assertRaisesRegex(PublicationError, "malformed JSON"):
                github_release("v0.1.0-alpha.2")

    def test_npm_scope_requires_an_administrative_role(self):
        for role, expected in (("owner", True), ("admin", True), ("developer", False)):
            commands = [
                SimpleNamespace(returncode=0, stdout=b"release-user\n", stderr=b""),
                SimpleNamespace(
                    returncode=0,
                    stdout=json.dumps({"release-user": role}).encode(),
                    stderr=b"",
                ),
            ]
            with self.subTest(role=role), patch.dict(
                "os.environ", {"NPM_TOKEN": "secret"}, clear=True
            ), patch("release.publication.run", side_effect=commands):
                self.assertEqual(credential_account()["npm"]["organizationAdmin"], expected)

    def test_scoped_registry_tokens_record_unprobeable_write_authorization(self):
        commands = [
            SimpleNamespace(returncode=0, stdout=b"release-user\n", stderr=b""),
            SimpleNamespace(
                returncode=0,
                stdout=json.dumps({"release-user": "owner"}).encode(),
                stderr=b"",
            ),
        ]
        environment = {
            "CARGO_REGISTRY_TOKEN": "cargo-secret",
            "NUGET_API_KEY": "nuget-secret",
            "NPM_TOKEN": "npm-secret",
            "GITHUB_TOKEN": "github-secret",
        }
        with patch.dict("os.environ", environment, clear=True), patch(
            "release.publication.request"
        ) as request, patch("release.publication.run", side_effect=commands) as run:
            account = credential_account()

        request.assert_not_called()
        self.assertEqual(run.call_count, 2)
        self.assertIsNone(account["cargo"]["authenticated"])
        self.assertIsNone(account["github"]["repositoryWrite"])
        self.assertTrue(account["ready"])

    def test_public_state_accounts_for_every_exact_registry_target(self):
        with tempfile.TemporaryDirectory() as temporary:
            retained, candidates, sums_path = self.retained_release(Path(temporary))
            account = public_release(retained, candidates)
            by_name = {subject["filename"]: subject for subject in account["subjects"]}
            support = {"SHA256SUMS": sums_path.read_bytes()}
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

            def package_result(subject, _version, *_metadata):
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
            ), patch(
                "release.publication.verify_nuget_repository_signature"
            ), patch(
                "release.publication.npm_registry_metadata",
                return_value={"dist-tags": {"alpha": account["version"]}},
            ), patch("release.publication.has_provenance", return_value=True):
                state = collect_state(account, sums_path)
            self.assertEqual(state["missingReleaseAssets"], [])
            self.assertEqual(len(state["targets"]), 10)
            self.assertEqual(
                {key: value["sha256"] for key, value in state["targets"].items()},
                {
                    key: value["sha256"]
                    for key, value in self.exact_registry_state(account)["targets"].items()
                },
            )
    def test_conflicting_release_support_asset_fails_before_planning(self):
        with tempfile.TemporaryDirectory() as temporary:
            retained, candidates, sums_path = self.retained_release(Path(temporary))
            account = public_release(retained, candidates)
            with patch(
                "release.publication.github_release",
                return_value={
                    "tag_name": account["tag"],
                    "prerelease": True,
                    "assets": [{"name": "SHA256SUMS", "url": "checksums"}],
                },
            ), patch("release.publication.github_asset_bytes", return_value=b"different"):
                with self.assertRaisesRegex(PublicationError, "support asset"):
                    collect_state(account, sums_path)

    def test_preflight_records_zero_writes_and_credentials_gate_publication(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            retained, candidates, sums_path = self.retained_release(directory)
            account = public_release(retained, candidates)
            exact = self.exact_public_state(account)
            arguments = argparse.Namespace(
                candidates=candidates,
                sums_out=sums_path,
                root=self.root,
                repository="azimuth-sh/azimuth",
                tag=account["tag"],
                revision=REVISION,
                state_out=directory / "state.json",
                plan_out=directory / "plan.json",
                receipt_out=directory / "preflight.json",
                require_credentials=False,
            )
            with patch(
                "release.publication.retained_tagged_release", return_value=retained
            ), patch("release.publication.collect_state", return_value=exact), patch(
                "release.publication.credential_account", return_value={"ready": False}
            ):
                preflight(arguments)
            receipt = json.loads(arguments.receipt_out.read_text())
            self.assertEqual(receipt["writes"], 0)
            self.assertEqual(receipt["artifactSetSha256"], hashlib.sha256(
                sums_path.read_bytes()
            ).hexdigest())
            self.assertEqual(receipt["plan"]["publish"], [])

            arguments.require_credentials = True
            with patch(
                "release.publication.retained_tagged_release", return_value=retained
            ), patch("release.publication.collect_state", return_value=exact), patch(
                "release.publication.credential_account", return_value={"ready": False}
            ):
                with self.assertRaisesRegex(PublicationError, "not ready"):
                    preflight(arguments)

    def test_publish_consumes_only_the_planner_selected_absent_target(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            retained, candidates, sums_path = self.retained_release(directory)
            account = public_release(retained, candidates)
            state = self.exact_public_state(account)
            selected = next(
                subject["key"]
                for subject in account["subjects"]
                if subject.get("ecosystem") != "npm"
            )
            del state["targets"][selected]
            state.update({"releaseExists": True, "missingReleaseAssets": []})
            state_path = directory / "state.json"
            plan_path = directory / "plan.json"
            state_path.write_text(json.dumps(state))
            plan_path.write_text(json.dumps(registry_publication_plan(account, state)))
            arguments = argparse.Namespace(
                candidates=candidates,
                tag=account["tag"],
                revision=REVISION,
                sums=sums_path,
                root=self.root,
                repository="azimuth-sh/azimuth",
                state=state_path,
                plan=plan_path,
                out=directory / "result.json",
            )
            with patch(
                "release.publication.retained_tagged_release", return_value=retained
            ), patch(
                "release.publication.credential_account", return_value={"ready": True}
            ), patch("release.publication.publish_target") as publish_one:
                publish(arguments)
            self.assertEqual(publish_one.call_args.args[0]["key"], selected)
            self.assertEqual(json.loads(arguments.out.read_text())["published"], [selected])

    def test_crates_upload_body_contains_the_exact_retained_archive(self):
        with tempfile.TemporaryDirectory() as temporary:
            archive = Path(temporary) / "azimuth-0.1.0-alpha.2.crate"
            manifest = (
                "[package]\nname='azimuth'\nversion='0.1.0-alpha.2'\n"
                "description='test'\nlicense='Apache-2.0'\nreadme='README.md'\n"
            ).encode()
            with tarfile.open(archive, "w:gz") as package:
                for name, content in (
                    ("azimuth-0.1.0-alpha.2/Cargo.toml", manifest),
                    ("azimuth-0.1.0-alpha.2/README.md", b"readme"),
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

    def test_npm_publication_uses_the_release_channel_as_its_dist_tag(self):
        subject = {"kind": "package", "ecosystem": "npm"}
        archive = Path("azimuth-sh-annotations-0.1.0-alpha.2.tgz")
        for version, tag in (
            ("0.1.0-alpha.2", "alpha"),
            ("0.2.0-rc.2", "rc"),
            ("1.0.0", "latest"),
        ):
            with self.subTest(version=version), patch.dict(
                "os.environ", {"NPM_TOKEN": "secret"}, clear=True
            ), patch("release.publication.run") as publish_command:
                publish_target(subject, archive, {"version": version})

            command = publish_command.call_args.args[0]
            self.assertEqual(command[-2:], ["--tag", tag])
            self.assertIn("--provenance", command)

    def test_first_npm_prerelease_accepts_the_registry_required_latest_tag(self):
        with tempfile.TemporaryDirectory() as temporary:
            retained, candidates, _ = self.retained_release(Path(temporary))
            account = public_release(retained, candidates)
            subject = next(
                item for item in account["subjects"] if item.get("ecosystem") == "npm"
            )
            state = self.exact_public_state(account)
            state["targets"][subject["key"]]["distTags"]["latest"] = account["version"]

            plan = registry_publication_plan(account, state)
            self.assertEqual(plan["publish"], [])
            self.assertEqual(plan["normalizeNpmTags"], [])
            validate_registry_completion(account, state)

    def test_stale_latest_on_an_earlier_prerelease_is_normalized_forward(self):
        # The alpha.1 -> alpha.2 case: publishing a second prerelease leaves `latest` behind on the
        # first, so a plain `npm install` would keep resolving the older alpha.
        with tempfile.TemporaryDirectory() as temporary:
            retained, candidates, _ = self.retained_release(Path(temporary))
            account = public_release(retained, candidates)
            state = self.exact_public_state(account)
            npm_subjects = [
                item for item in account["subjects"] if item.get("ecosystem") == "npm"
            ]
            self.assertTrue(npm_subjects)
            for subject in npm_subjects:
                state["targets"][subject["key"]]["distTags"]["latest"] = "0.1.0-alpha.1"

            plan = registry_publication_plan(account, state)
            self.assertEqual(plan["publish"], [])
            self.assertEqual(
                plan["normalizeNpmTags"], sorted(item["key"] for item in npm_subjects)
            )
            with self.assertRaisesRegex(PublicationError, "npm dist-tag drift"):
                validate_registry_completion(account, state)

    def test_normalization_moves_latest_forward_when_no_stable_version_exists(self):
        subject = {
            "key": "package:typescript-annotations",
            "identity": "@azimuth-sh/annotations",
        }
        version = "0.1.0-alpha.2"
        results = [
            SimpleNamespace(stdout=b"alpha: 0.1.0-alpha.1\nlatest: 0.1.0-alpha.1\n"),
            SimpleNamespace(stdout=b""),
            SimpleNamespace(stdout=b""),
            SimpleNamespace(stdout=f"alpha: {version}\nlatest: {version}\n".encode()),
        ]
        with patch.dict("os.environ", {"NPM_TOKEN": "secret"}, clear=True), patch(
            "release.publication.run", side_effect=results
        ) as npm:
            normalize_npm_dist_tags(subject, version, [])
        moved = [
            call.args[0]
            for call in npm.call_args_list
            if call.args[0][:3] == ["npm", "dist-tag", "add"]
        ]
        self.assertEqual(
            moved,
            [
                ["npm", "dist-tag", "add", f"@azimuth-sh/annotations@{version}", "alpha"],
                ["npm", "dist-tag", "add", f"@azimuth-sh/annotations@{version}", "latest"],
            ],
        )

    def test_npm_latest_prerelease_with_a_stable_version_blocks_completion(self):
        with tempfile.TemporaryDirectory() as temporary:
            retained, candidates, _ = self.retained_release(Path(temporary))
            account = public_release(retained, candidates)
            subject = next(
                item for item in account["subjects"] if item.get("ecosystem") == "npm"
            )
            state = self.exact_public_state(account)
            target = state["targets"][subject["key"]]
            target["distTags"]["latest"] = account["version"]
            target["stableVersions"] = ["0.0.1"]

            plan = registry_publication_plan(account, state)
            self.assertEqual(plan["publish"], [])
            self.assertEqual(plan["normalizeNpmTags"], [subject["key"]])
            with self.assertRaisesRegex(PublicationError, "npm dist-tag drift"):
                validate_registry_completion(account, state)

            absent = self.exact_public_state(account)
            del absent["targets"][subject["key"]]
            plan = registry_publication_plan(account, absent)
            self.assertEqual(plan["publish"], [subject["key"]])
            self.assertEqual(plan["normalizeNpmTags"], [subject["key"]])

    def test_npm_tag_normalization_preserves_required_first_latest_tag(self):
        subject = {
            "key": "package:typescript-annotations",
            "identity": "@azimuth-sh/annotations",
        }
        version = "0.1.0-alpha.2"
        results = [
            SimpleNamespace(stdout=f"alpha: {version}\nlatest: {version}\n".encode()),
            SimpleNamespace(stdout=f"alpha: {version}\nlatest: {version}\n".encode()),
        ]
        with patch.dict("os.environ", {"NPM_TOKEN": "secret"}, clear=True), patch(
            "release.publication.run", side_effect=results
        ) as npm:
            normalize_npm_dist_tags(subject, version, [])

        commands = [call.args[0] for call in npm.call_args_list]
        self.assertEqual(len(commands), 2)
        self.assertTrue(all(command[:3] == ["npm", "dist-tag", "ls"] for command in commands))

    def test_npm_tag_normalization_does_not_guess_a_stable_latest_target(self):
        subject = {
            "key": "package:typescript-annotations",
            "identity": "@azimuth-sh/annotations",
        }
        version = "0.1.0-alpha.2"
        result = SimpleNamespace(stdout=f"alpha: {version}\nlatest: {version}\n".encode())
        with patch.dict("os.environ", {"NPM_TOKEN": "secret"}, clear=True), patch(
            "release.publication.run", return_value=result
        ) as npm, self.assertRaisesRegex(PublicationError, "intended stable target"):
            normalize_npm_dist_tags(subject, version, ["0.0.1"])

        self.assertEqual(len(npm.call_args_list), 1)

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
        subject = {"key": "image:api", "identity": "ghcr.io/example/api"}
        result = type("Result", (), {"returncode": 0, "stdout": raw, "stderr": b""})()
        with patch("release.publication.run", return_value=result) as inspect:
            checksum, platforms = image_manifest(subject, "0.1.0-alpha.2")
        self.assertEqual(checksum, hashlib.sha256(raw).hexdigest())
        self.assertEqual(platforms, ["linux/amd64", "linux/arm64"])
        self.assertEqual(
            inspect.call_args.args[0],
            [
                "skopeo",
                "inspect",
                "--no-creds",
                "--raw",
                "docker://ghcr.io/example/api:0.1.0-alpha.2",
            ],
        )

    def test_image_state_verifies_the_complete_retained_release(self):
        root = Path("/requested/checkout")
        candidates = Path("/retained/candidates")
        retained = {"version": "0.1.0-alpha.2"}
        account = {
            "version": "0.1.0-alpha.2",
            "subjects": [
                {
                    "kind": "image",
                    "id": "api",
                    "identity": "ghcr.io/example/api",
                    "registryDigest": "sha256:" + "b" * 64,
                    "platforms": ["linux/amd64", "linux/arm64"],
                }
            ],
        }
        arguments = argparse.Namespace(
            candidates=candidates,
            root=root,
            tag="v0.1.0-alpha.2",
            revision=REVISION,
            sums=Path("/retained/SHA256SUMS"),
            id="api",
        )
        with patch(
            "release.publication.retained_tagged_release", return_value=retained
        ) as load_release, patch(
            "release.publication.verify_checksums"
        ), patch(
            "release.publication.public_release", return_value=account
        ), patch(
            "release.publication.image_manifest",
            return_value=("b" * 64, ["linux/amd64", "linux/arm64"]),
        ):
            image_state(arguments)
        load_release.assert_called_once_with(candidates, REVISION, "v0.1.0-alpha.2", root)

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

    def test_image_provenance_chains_candidate_and_publication_revisions(self):
        subject = {"kind": "image", "retainedSha256": "a" * 64}
        candidate_revision = "b" * 40
        publication_revision = "c" * 40
        evidence = {
            (subject["retainedSha256"], candidate_revision),
            ("d" * 64, publication_revision),
        }
        with patch(
            "release.publication.has_provenance",
            side_effect=lambda checksum, revision, _repository: (checksum, revision) in evidence,
        ):
            self.assertEqual(
                subject_provenance(
                    subject,
                    "d" * 64,
                    candidate_revision,
                    publication_revision,
                    "azimuth-sh/azimuth",
                ),
                (True, "retained-to-published"),
            )
            self.assertEqual(
                subject_provenance(
                    subject,
                    "e" * 64,
                    candidate_revision,
                    publication_revision,
                    "azimuth-sh/azimuth",
                ),
                (False, None),
            )

    def test_publication_workflow_builds_and_publishes_from_the_tag_run(self):
        structure = publication_workflow_structure(self.root)
        self.assertEqual(structure["trigger"], "tag-push")
        self.assertEqual(structure["artifactBuilds"], "same-run")
        source = (self.root / ".github/workflows/publish.yml").read_text()
        self.assertEqual(source.count("uses: ./.github/workflows/ci.yml"), 1)
        self.assertNotIn("run-id:", source)

    def test_publication_workflow_scopes_tokens_and_supplies_complete_image_inputs(self):
        source = (self.root / ".github/workflows/publish.yml").read_text()
        self.assertEqual(source.count("persist-credentials: false"), 3)
        self.assertEqual(source.count("docker/login-action@v4"), 2)
        self.assertEqual(source.count("attestations: write"), 3)
        self.assertIn("permissions:\n  contents: read\n  id-token: write", source)
        self.assertEqual(source.count("pattern: candidates-*"), 3)

    def test_publication_workflow_fails_static_credential_and_provenance_mutations(self):
        source = (self.root / ".github/workflows/publish.yml").read_text()
        mutations = {
            "credential": source.replace("            --require-credentials", ""),
            "cargo-secret": source.replace(
                "CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}",
                "CARGO_REGISTRY_TOKEN: ''",
            ),
            "github-release-permission": source.replace(
                "      contents: write", "      contents: read", 1
            ),
            "provenance": source.replace("push-to-registry: true", "push-to-registry: false"),
            "build": source.replace(
                "uses: ./.github/workflows/ci.yml",
                "uses: octo-org/example/.github/workflows/ci.yml@main",
            ),
        }
        for name, changed in mutations.items():
            with self.subTest(name=name), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                workflow = root / ".github/workflows"
                workflow.mkdir(parents=True)
                (workflow / "publish.yml").write_text(changed)
                with self.assertRaises(PublicationError):
                    publication_workflow_structure(root)

    def test_qualification_declares_realization_without_fabricating_operational_evidence(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary)
            qualify(argparse.Namespace(root=self.root, out=output))
            linkage = json.loads((output / "publication-linkage.json").read_text())
            qualification = json.loads((output / "publication.json").read_text())
            self.assertEqual(
                {
                    (entry["claim"], entry["site"], entry["file"])
                    for entry in linkage["realizes"]
                },
                {
                    (
                        "tagged-candidates-are-verifiable",
                        "public_release_preflight",
                        "release/publication.py",
                    ),
                    (
                        "tagged-candidates-are-verifiable",
                        "tagged_release_artifact_validator",
                        "release/publication.py",
                    ),
                    (
                        "tagged-candidates-are-verifiable",
                        "published_image_attestation",
                        ".github/workflows/publish.yml",
                    ),
                    (
                        "partial-publication-resumes-safely",
                        "public_registry_adapters",
                        "release/publication.py",
                    ),
                    (
                        "partial-publication-resumes-safely",
                        "public_completion_gate",
                        "release/publication.py",
                    ),
                },
            )
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
            self.assertEqual(qualification["operationalEvidence"], "pending")


if __name__ == "__main__":
    unittest.main()
