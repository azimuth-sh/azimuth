#!/usr/bin/env python3

import argparse
import base64
import copy
import datetime
import hashlib
import json
import os
import re
import struct
import subprocess
import tarfile
import tomllib
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

if __package__:
    from .orchestrate import (
        OrchestrationError,
        combined_digest,
        digest,
        file_index,
        plan_publication,
        read_json,
        source_entry,
        validate_completion,
        verify,
        verify_tag,
        workflow_jobs,
    )
else:
    from orchestrate import (
        OrchestrationError,
        combined_digest,
        digest,
        file_index,
        plan_publication,
        read_json,
        source_entry,
        validate_completion,
        verify,
        verify_tag,
        workflow_jobs,
    )


ROOT = Path(__file__).resolve().parent.parent
REPOSITORY = "azimuth-sh/azimuth"
USER_AGENT = "azimuth-release/0.1.0-alpha.1 (https://github.com/azimuth-sh/azimuth)"
SUPPORT_ASSETS = ("candidates.json", "SHA256SUMS")
PUBLICATION_WORKFLOW = ROOT / ".github/workflows/publish.yml"


class PublicationError(Exception):
    pass


def require(condition, message):
    if not condition:
        raise PublicationError(message)


def run(command, *, input_bytes=None, check=True, env=None, cwd=None):
    result = subprocess.run(
        [str(item) for item in command],
        input=input_bytes,
        capture_output=True,
        check=False,
        env=env,
        cwd=None if cwd is None else str(cwd),
    )
    if check and result.returncode != 0:
        message = result.stderr.decode(errors="replace").strip()
        raise PublicationError(f"command failed ({command[0]}): {message}")
    return result


def run_json(command, *, allow_not_found=False, env=None):
    result = run(command, check=False, env=env)
    if result.returncode != 0:
        error = result.stderr.decode(errors="replace")
        if allow_not_found and ("HTTP 404" in error or "not found" in error.lower()):
            return None
        raise PublicationError(f"command failed ({command[0]}): {error.strip()}")
    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise PublicationError(f"{command[0]} returned malformed JSON") from error


def request(url, *, headers=None, body=None, method=None):
    request_headers = {"User-Agent": USER_AGENT, **(headers or {})}
    call = urllib.request.Request(url, data=body, headers=request_headers, method=method)
    try:
        with urllib.request.urlopen(call, timeout=30) as response:
            return response.status, dict(response.headers), response.read()
    except urllib.error.HTTPError as error:
        return error.code, dict(error.headers), error.read()
    except urllib.error.URLError as error:
        raise PublicationError(f"registry request failed for {url}: {error.reason}") from error


def now():
    return datetime.datetime.now(datetime.UTC).replace(microsecond=0).isoformat()


def candidate_path(candidate_root, subject):
    matches = file_index(candidate_root).get(subject["filename"], [])
    require(len(matches) == 1, f"{subject['key']}: retained file population differs")
    return matches[0]


def oci_index_digest(path):
    try:
        with tarfile.open(path) as archive:
            index = json.load(archive.extractfile("index.json"))
            descriptors = index.get("manifests", [])
            require(len(descriptors) == 1, "OCI archive must contain one tagged index")
            value = descriptors[0].get("digest", "")
            require(
                re.fullmatch(r"sha256:[0-9a-f]{64}", value) is not None,
                "OCI index digest is invalid",
            )
            algorithm, checksum = value.split(":", 1)
            blob = archive.extractfile(f"blobs/{algorithm}/{checksum}").read()
    except (KeyError, OSError, tarfile.TarError, json.JSONDecodeError) as error:
        raise PublicationError(f"cannot inspect OCI archive {path}: {error}") from error
    require(hashlib.sha256(blob).hexdigest() == checksum, "OCI index blob checksum differs")
    return value


def public_account(retained_account, candidate_root):
    account = copy.deepcopy(retained_account)
    for subject in account["subjects"]:
        if subject["kind"] != "image":
            continue
        retained_checksum = subject["sha256"]
        registry_checksum = oci_index_digest(candidate_path(candidate_root, subject))
        subject["retainedSha256"] = retained_checksum
        subject["sha256"] = registry_checksum.removeprefix("sha256:")
        subject["registryDigest"] = registry_checksum
    return account


def annotated_tag_revision(root, tag):
    kind = run(["git", "cat-file", "-t", tag], cwd=root).stdout.decode().strip()
    require(kind == "tag", f"tag {tag!r} is not annotated")
    revision = run(["git", "rev-list", "-n", "1", tag], cwd=root).stdout.decode().strip()
    require(re.fullmatch(r"[0-9a-f]{40}", revision) is not None, "tag revision is invalid")
    return revision


def publication_run_commands(job):
    lines = job.splitlines()
    commands = []
    index = 0
    while index < len(lines):
        match = re.match(r"^(\s*)(?:-\s+)?run:\s*(.*?)\s*$", lines[index])
        if match is None:
            index += 1
            continue
        value = match.group(2)
        if value not in ("|", "|-", ">", ">-"):
            commands.append(value)
            index += 1
            continue
        block = []
        index += 1
        content_indent = None
        while index < len(lines):
            line = lines[index]
            indent = len(line) - len(line.lstrip())
            if line.strip() and content_indent is None:
                content_indent = indent
            if line.strip() and indent < content_indent:
                break
            block.append(line[content_indent:] if line.strip() else "")
            index += 1
        commands.append("\n".join(block) if value.startswith("|") else " ".join(block))
    return commands


def publication_workflow_account(root=ROOT):
    source = (Path(root) / PUBLICATION_WORKFLOW.relative_to(ROOT)).read_text()
    trigger = source.split("permissions:", 1)[0]
    require("workflow_dispatch:" in trigger, "publication workflow has no owner dispatch")
    require("pull_request:" not in trigger, "publication workflow runs on pull requests")
    require("push:" not in trigger, "publication workflow publishes from a push trigger")
    jobs = workflow_jobs(source)
    for name in ("publish", "image-provenance", "complete"):
        require(name in jobs, f"publication workflow omits {name!r}")
        require("environment: release" in jobs[name], f"publication job {name!r} is unbounded")
    for secret in ("CARGO_REGISTRY_TOKEN", "NPM_TOKEN", "NUGET_API_KEY"):
        require(
            f"{secret}: ${{{{ secrets.{secret} }}}}" in jobs["publish"],
            f"publication workflow omits the {secret} release secret",
        )
    require("GH_TOKEN: ${{ github.token }}" in jobs["publish"],
            "publication workflow omits the bounded GitHub token")
    require("contents: write" in jobs["publish"],
            "publication workflow cannot write the GitHub Release")
    require("packages: write" in jobs["publish"],
            "publication workflow cannot write GHCR")
    require("actions/download-artifact@v8" in source, "publication omits retained downloads")
    require("run-id: ${{ inputs.rehearsal_run_id }}" in source,
            "publication does not bind the rehearsal run")
    require("release/candidates.py" not in source, "publication rebuilds candidates")
    require("docker/build-push-action" not in source, "publication rebuilds images")
    publish_commands = publication_run_commands(jobs["publish"])
    commands = "\n".join(publish_commands)
    require("publication.py preflight" in commands, "publication omits preflight")
    require("--require-credentials" in commands, "publication omits the credential gate")
    require("publication.py publish" in commands, "publication omits the selected write step")
    require(
        commands.index("publication.py preflight") < commands.index("publication.py publish"),
        "publication writes before preflight",
    )
    for expression in ("${{ inputs.", "${{ matrix."):
        require(
            not any(expression in command for command in publish_commands),
            "publication expands untrusted values directly in a shell command",
        )
    require("subject-digest:" in jobs["image-provenance"],
            "published image provenance has no digest subject")
    require("push-to-registry: true" in jobs["image-provenance"],
            "published image provenance is not attached to GHCR")
    require("needs: [publish, image-provenance]" in jobs["complete"],
            "completion does not wait for publication and provenance")
    return {
        "trigger": "workflow_dispatch",
        "candidateBuilds": 0,
        "jobs": ["publish", "image-provenance", "complete"],
    }


def attestation_payloads(checksum, repository=REPOSITORY):
    response = run_json(
        ["gh", "api", f"repos/{repository}/attestations/sha256:{checksum}"],
        allow_not_found=True,
    )
    if response is None:
        return []
    payloads = []
    for attestation in response.get("attestations", []):
        encoded = attestation.get("bundle", {}).get("dsseEnvelope", {}).get("payload", "")
        try:
            payloads.append(json.loads(base64.b64decode(encoded)))
        except (ValueError, json.JSONDecodeError):
            continue
    return payloads


def has_provenance(checksum, revision, repository=REPOSITORY):
    for payload in attestation_payloads(checksum, repository):
        subjects = payload.get("subject", [])
        dependencies = payload.get("predicate", {}).get("buildDefinition", {}).get(
            "resolvedDependencies", []
        )
        exact_subject = any(
            subject.get("digest", {}).get("sha256") == checksum for subject in subjects
        )
        exact_revision = any(
            dependency.get("digest", {}).get("gitCommit") == revision
            for dependency in dependencies
        )
        if exact_subject and exact_revision:
            return True
    return False


def package_url(subject, version, metadata=None):
    identity = subject["identity"]
    if subject["ecosystem"] == "cargo":
        name = urllib.parse.quote(identity, safe="")
        return f"https://static.crates.io/crates/{name}/{name}-{version}.crate"
    if subject["ecosystem"] == "nuget":
        name = identity.lower()
        return f"https://api.nuget.org/v3-flatcontainer/{name}/{version}/{name}.{version}.nupkg"
    require(metadata is not None, f"{subject['key']}: npm metadata is required")
    versions = metadata.get("versions", {})
    return versions.get(version, {}).get("dist", {}).get("tarball")


def package_bytes(subject, version):
    if subject["ecosystem"] == "cargo":
        name = subject["identity"].lower()
        if len(name) == 1:
            relative = f"1/{name}"
        elif len(name) == 2:
            relative = f"2/{name}"
        elif len(name) == 3:
            relative = f"3/{name[0]}/{name}"
        else:
            relative = f"{name[:2]}/{name[2:4]}/{name}"
        status, _, content = request(f"https://index.crates.io/{relative}")
        if status == 404:
            return None, None
        require(status == 200, f"{subject['key']}: Cargo index returned HTTP {status}")
        try:
            versions = [json.loads(line) for line in content.splitlines() if line]
        except json.JSONDecodeError as error:
            raise PublicationError(f"{subject['key']}: Cargo index is malformed") from error
        if not any(item.get("vers") == version for item in versions):
            return None, None
        url = package_url(subject, version)
    elif subject["ecosystem"] == "npm":
        identity = urllib.parse.quote(subject["identity"], safe="")
        status, _, content = request(f"https://registry.npmjs.org/{identity}")
        if status == 404:
            return None, None
        require(status == 200, f"{subject['key']}: npm state returned HTTP {status}")
        try:
            metadata = json.loads(content)
        except json.JSONDecodeError as error:
            raise PublicationError(f"{subject['key']}: npm state is malformed") from error
        url = package_url(subject, version, metadata)
        if url is None:
            return None, None
        location = urllib.parse.urlsplit(url)
        require(
            location.scheme == "https" and location.hostname == "registry.npmjs.org",
            f"{subject['key']}: npm tarball URL is not an npm registry HTTPS URL",
        )
    else:
        url = package_url(subject, version)
    status, _, content = request(url)
    if status == 404:
        return None, url
    require(status == 200, f"{subject['key']}: package download returned HTTP {status}")
    return content, url


def github_release(tag, repository=REPOSITORY):
    return run_json(
        ["gh", "api", f"repos/{repository}/releases/tags/{tag}"],
        allow_not_found=True,
    )


def github_asset_bytes(asset):
    result = run(
        ["gh", "api", "-H", "Accept: application/octet-stream", asset["url"]]
    )
    return result.stdout


def image_manifest(subject, version):
    result = run(
        ["skopeo", "inspect", "--raw", f"docker://{subject['identity']}:{version}"],
        check=False,
    )
    if result.returncode != 0:
        error = result.stderr.decode(errors="replace").lower()
        absent = ("manifest unknown", "name unknown", "not found", "status code: 404")
        if any(marker in error for marker in absent):
            return None
        raise PublicationError(f"{subject['key']}: image state failed: {error.strip()}")
    try:
        manifest = json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise PublicationError(f"{subject['key']}: image manifest is malformed") from error
    checksum = hashlib.sha256(result.stdout).hexdigest()
    platforms = sorted(
        f"{item['platform']['os']}/{item['platform']['architecture']}"
        for item in manifest.get("manifests", [])
        if item.get("platform", {}).get("os") not in (None, "unknown")
        and item.get("platform", {}).get("architecture") not in (None, "unknown")
    )
    return checksum, platforms


def support_asset_account(account_path, sums_path):
    return {
        "candidates.json": digest(account_path),
        "SHA256SUMS": digest(sums_path),
    }


def collect_state(account, account_path, sums_path, repository=REPOSITORY):
    version = account["version"]
    revision = account["revision"]
    state = {
        "format": "azimuth-public-registry-state",
        "schemaVersion": 1,
        "observedAt": now(),
        "tag": account["tag"],
        "version": version,
        "revision": revision,
        "targets": {},
        "releaseExists": False,
        "missingReleaseAssets": list(SUPPORT_ASSETS),
    }
    release = github_release(account["tag"], repository)
    assets = {}
    if release is not None:
        require(release.get("tag_name") == account["tag"], "GitHub Release tag differs")
        require(release.get("prerelease") is True, "GitHub Release is not a prerelease")
        state["releaseExists"] = True
        assets = {asset["name"]: asset for asset in release.get("assets", [])}
        expected_support = support_asset_account(account_path, sums_path)
        missing = []
        for name, checksum in expected_support.items():
            if name not in assets:
                missing.append(name)
                continue
            require(
                hashlib.sha256(github_asset_bytes(assets[name])).hexdigest() == checksum,
                f"GitHub Release support asset {name!r} conflicts",
            )
        state["missingReleaseAssets"] = missing

    for subject in account["subjects"]:
        observed = None
        url = None
        if subject["kind"] == "package":
            content, url = package_bytes(subject, version)
            if content is not None:
                observed = hashlib.sha256(content).hexdigest()
        elif subject["kind"] == "native":
            asset = assets.get(subject["filename"])
            if asset is not None:
                content = github_asset_bytes(asset)
                observed = hashlib.sha256(content).hexdigest()
                url = asset.get("browser_download_url")
        else:
            image = image_manifest(subject, version)
            if image is not None:
                observed, platforms = image
                url = f"https://ghcr.io/{subject['identity'].removeprefix('ghcr.io/')}:{version}"
        if observed is None:
            continue
        target = {
            "identity": subject["identity"],
            "sha256": observed,
            "provenance": has_provenance(observed, revision, repository),
            "url": url,
        }
        if subject["kind"] == "image":
            target["platforms"] = platforms
        state["targets"][subject["key"]] = target
    return state


def credential_account():
    cargo_token = os.environ.get("CARGO_REGISTRY_TOKEN", "")
    nuget_token = os.environ.get("NUGET_API_KEY", "")
    npm_token = os.environ.get("NPM_TOKEN", "")
    github_token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN", "")

    npm_identity = None
    npm_scope = False
    if npm_token:
        npm_env = {**os.environ, "NODE_AUTH_TOKEN": npm_token}
        identity = run(["npm", "whoami"], check=False, env=npm_env)
        if identity.returncode == 0:
            npm_identity = identity.stdout.decode().strip()
            membership = run(
                ["npm", "org", "ls", "azimuth-sh", "--json"],
                check=False,
                env=npm_env,
            )
            if membership.returncode == 0:
                try:
                    roster = json.loads(membership.stdout)
                    npm_scope = (
                        isinstance(roster, dict)
                        and roster.get(npm_identity) in ("owner", "admin")
                    )
                except json.JSONDecodeError:
                    npm_scope = False

    result = {
        "cargo": {
            "configured": bool(cargo_token),
            "authenticated": None,
            "limitation": (
                "A publish-new scoped token has no non-publishing identity probe; "
                "authorization is first observed by the registry write"
            ),
        },
        "nuget": {
            "configured": bool(nuget_token),
            "authenticated": None,
            "limitation": "NuGet exposes no non-publishing API-key authorization probe",
        },
        "npm": {
            "configured": bool(npm_token),
            "identity": npm_identity,
            "organizationAdmin": npm_scope,
        },
        "github": {
            "configured": bool(github_token),
            "repositoryWrite": None,
            "packageWrite": None,
            "limitation": (
                "GitHub Release and GHCR write permission are first observed by their "
                "registry writes; the workflow declaration bounds both tokens"
            ),
        },
    }
    result["ready"] = (
        result["cargo"]["configured"]
        and result["nuget"]["configured"]
        and result["npm"]["organizationAdmin"]
        and result["github"]["configured"]
    )
    return result


def write_json(path, value):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + "\n")


def preflight(arguments):
    account_path = Path(arguments.account)
    retained_account = verify(read_json(account_path), arguments.candidates, arguments.root)
    account = public_account(retained_account, arguments.candidates)
    require(arguments.tag == account["tag"], "requested tag differs from retained account")
    verify_tag(arguments.root, arguments.tag, account["revision"])
    require(
        annotated_tag_revision(arguments.root, arguments.tag) == account["revision"],
        "annotated tag revision differs from retained account",
    )
    require(arguments.run_revision == account["revision"], "rehearsal run revision differs")
    state = collect_state(account, account_path, arguments.sums, arguments.repository)
    plan = plan_publication(account, state)
    credentials = credential_account()
    receipt = {
        "format": "azimuth-publication-preflight",
        "schemaVersion": 1,
        "observedAt": now(),
        "tag": account["tag"],
        "revision": account["revision"],
        "rehearsalRun": arguments.rehearsal_run,
        "candidateAccountSha256": digest(account_path),
        "stateSha256": None,
        "plan": plan,
        "credentials": credentials,
        "writes": 0,
    }
    write_json(arguments.state_out, state)
    receipt["stateSha256"] = digest(arguments.state_out)
    write_json(arguments.plan_out, plan)
    write_json(arguments.receipt_out, receipt)
    if arguments.require_credentials:
        require(credentials["ready"], "publication credentials are not ready")
    print(
        f"preflight: publish={len(plan['publish'])} preserve={len(plan['preserve'])} "
        f"credentials_ready={str(credentials['ready']).lower()} writes=0"
    )


def crate_metadata(archive):
    with tarfile.open(archive, "r:gz") as package:
        manifest_name = next(
            member.name
            for member in package.getmembers()
            if member.isfile() and member.name.endswith("/Cargo.toml")
        )
        manifest = tomllib.loads(package.extractfile(manifest_name).read().decode())
        metadata = manifest["package"]
        readme_name = metadata.get("readme")
        readme = None
        if readme_name:
            root = manifest_name.rsplit("/", 1)[0]
            readme = package.extractfile(f"{root}/{readme_name}").read().decode()
    return {
        "name": metadata["name"],
        "vers": metadata["version"],
        "deps": [],
        "features": manifest.get("features", {}),
        "authors": metadata.get("authors", []),
        "description": metadata.get("description"),
        "documentation": metadata.get("documentation"),
        "homepage": metadata.get("homepage"),
        "readme": readme,
        "readme_file": readme_name,
        "keywords": metadata.get("keywords", []),
        "categories": metadata.get("categories", []),
        "license": metadata.get("license"),
        "license_file": metadata.get("license-file"),
        "repository": metadata.get("repository"),
        "badges": {},
        "links": metadata.get("links"),
        "rust_version": metadata.get("rust-version"),
    }


def publish_crate(archive):
    token = os.environ.get("CARGO_REGISTRY_TOKEN", "")
    require(token, "CARGO_REGISTRY_TOKEN is absent")
    archive_bytes = Path(archive).read_bytes()
    metadata = json.dumps(crate_metadata(archive), separators=(",", ":")).encode()
    body = (
        struct.pack("<I", len(metadata))
        + metadata
        + struct.pack("<I", len(archive_bytes))
        + archive_bytes
    )
    status, _, response = request(
        "https://crates.io/api/v1/crates/new",
        headers={
            "Accept": "application/json",
            "Authorization": token,
            "Content-Type": "application/octet-stream",
        },
        body=body,
        method="PUT",
    )
    require(
        200 <= status < 300,
        f"crates.io publication returned HTTP {status}: {response[:200]!r}",
    )
    try:
        result = json.loads(response)
    except json.JSONDecodeError as error:
        raise PublicationError("crates.io publication returned malformed JSON") from error
    require(not result.get("errors"), f"crates.io rejected publication: {result.get('errors')}")


def ensure_github_release(account, state, account_path, sums_path, repository=REPOSITORY):
    if not state["releaseExists"]:
        run(
            [
                "gh", "release", "create", account["tag"], "--repo", repository,
                "--verify-tag", "--prerelease", "--title", f"Azimuth {account['version']}",
                "--notes", "First public alpha for Azimuth dogfooding.",
            ]
        )
    support = {"candidates.json": account_path, "SHA256SUMS": sums_path}
    for name in state["missingReleaseAssets"]:
        run(["gh", "release", "upload", account["tag"], support[name], "--repo", repository])


def publish_target(subject, path, account, repository=REPOSITORY):
    if subject["kind"] == "package":
        if subject["ecosystem"] == "cargo":
            publish_crate(path)
        elif subject["ecosystem"] == "nuget":
            token = os.environ.get("NUGET_API_KEY", "")
            require(token, "NUGET_API_KEY is absent")
            run(
                [
                    "dotnet", "nuget", "push", path, "--api-key", token,
                    "--source", "https://api.nuget.org/v3/index.json",
                ]
            )
        else:
            token = os.environ.get("NPM_TOKEN", "")
            require(token, "NPM_TOKEN is absent")
            env = {**os.environ, "NODE_AUTH_TOKEN": token}
            run(["npm", "publish", path, "--access", "public", "--provenance"], env=env)
    elif subject["kind"] == "native":
        run(
            [
                "gh", "release", "upload", account["tag"], path,
                "--repo", repository,
            ]
        )
    else:
        run(
            [
                "skopeo", "copy", "--all", f"oci-archive:{path}",
                f"docker://{subject['identity']}:{account['version']}",
            ]
        )


def publish(arguments):
    account_path = Path(arguments.account)
    retained_account = verify(read_json(account_path), arguments.candidates, arguments.root)
    account = public_account(retained_account, arguments.candidates)
    state = read_json(arguments.state)
    supplied_plan = read_json(arguments.plan)
    expected_plan = plan_publication(account, state)
    require(supplied_plan == expected_plan, "publication plan is stale or modified")
    credentials = credential_account()
    require(credentials["ready"], "publication credentials are not ready")
    subjects = {subject["key"]: subject for subject in account["subjects"]}
    native_selected = any(subjects[key]["kind"] == "native" for key in supplied_plan["publish"])
    if native_selected or state["missingReleaseAssets"]:
        ensure_github_release(account, state, account_path, arguments.sums, arguments.repository)
    published = []
    for key in supplied_plan["publish"]:
        subject = subjects[key]
        publish_target(
            subject,
            candidate_path(arguments.candidates, subject),
            account,
            arguments.repository,
        )
        published.append(key)
    result = {
        "format": "azimuth-publication-result",
        "schemaVersion": 1,
        "tag": account["tag"],
        "revision": account["revision"],
        "published": published,
        "preserved": supplied_plan["preserve"],
    }
    write_json(arguments.out, result)
    print(f"publication writes completed: {len(published)} target(s)")


def image_state(arguments):
    retained_account = verify(read_json(arguments.account), arguments.candidates, arguments.root)
    account = public_account(retained_account, arguments.candidates)
    subject = next(
        (
            item
            for item in account["subjects"]
            if item["kind"] == "image" and item["id"] == arguments.id
        ),
        None,
    )
    require(subject is not None, f"image {arguments.id!r} is absent from the account")
    observed = image_manifest(subject, account["version"])
    require(observed is not None, f"image {arguments.id!r} is not public")
    checksum, platforms = observed
    require(checksum == subject["registryDigest"].removeprefix("sha256:"),
            f"image {arguments.id!r} registry digest differs")
    require(platforms == subject["platforms"], f"image {arguments.id!r} platforms differ")
    print(json.dumps({"name": subject["identity"], "digest": f"sha256:{checksum}"}))


def complete(arguments):
    account_path = Path(arguments.account)
    retained_account = verify(read_json(account_path), arguments.candidates, arguments.root)
    account = public_account(retained_account, arguments.candidates)
    state = collect_state(account, account_path, arguments.sums, arguments.repository)
    require(not state["missingReleaseAssets"], "GitHub Release support assets are incomplete")
    completion = validate_completion(account, state)
    receipt = {
        "format": "azimuth-public-release-completion",
        "schemaVersion": 1,
        "observedAt": now(),
        "tag": account["tag"],
        "revision": account["revision"],
        "rehearsalRun": arguments.rehearsal_run,
        "publicationRun": arguments.publication_run,
        "candidateAccountSha256": digest(account_path),
        "state": state,
        "completion": completion,
    }
    write_json(arguments.out, receipt)
    print(f"public release complete: targets={len(completion['targets'])}")


def qualify(arguments):
    root = Path(arguments.root)
    output = Path(arguments.out)
    workflow = root / PUBLICATION_WORKFLOW.relative_to(ROOT)
    implementation = root / "release/publication.py"
    workflow_account = publication_workflow_account(root)
    output.mkdir(parents=True, exist_ok=True)
    write_json(
        output / "publication.json",
        {
            "format": "azimuth-publication-qualification",
            "schemaVersion": 1,
            "workflow": workflow_account,
            "operationalEvidence": "pending",
        },
    )
    implementation_fingerprint = combined_digest([implementation])
    workflow_fingerprint = combined_digest([workflow])
    sites = {
        "tag-catalog-and-revision-agree": (
            "public_release_preflight",
            "release/publication.py",
            implementation_fingerprint,
        ),
        "retained-downloads-have-checksums": (
            "retained_candidate_verifier",
            "release/publication.py",
            implementation_fingerprint,
        ),
        "executable-subjects-have-provenance": (
            "published_image_attestation",
            ".github/workflows/publish.yml",
            workflow_fingerprint,
        ),
        "exact-existing-target-is-preserved": (
            "public_registry_adapters",
            "release/publication.py",
            implementation_fingerprint,
        ),
        "absent-target-is-selected": (
            "public_registry_adapters",
            "release/publication.py",
            implementation_fingerprint,
        ),
        "conflicting-target-fails": (
            "public_registry_adapters",
            "release/publication.py",
            implementation_fingerprint,
        ),
        "completion-needs-public-retrieval": (
            "public_completion_gate",
            "release/publication.py",
            implementation_fingerprint,
        ),
    }
    write_json(
        output / "publication-linkage.json",
        {
            "realizes": [
                source_entry(scenario, site, file, fingerprint)
                for scenario, (site, file, fingerprint) in sites.items()
            ],
            "covers": [],
            "mechanism_implementations": [],
            "mechanism_covers": [],
            "class_members": [],
            "enumerations": [],
            "artifacts": [],
            "observations": [],
        },
    )
    print("qualified owner-dispatched public publication with operational evidence pending")


def parser():
    root = argparse.ArgumentParser()
    root.add_argument("--root", type=Path, default=ROOT)
    root.add_argument("--repository", default=REPOSITORY)
    commands = root.add_subparsers(dest="command", required=True)

    preflight_parser = commands.add_parser("preflight")
    preflight_parser.add_argument("--account", type=Path, required=True)
    preflight_parser.add_argument("--candidates", type=Path, required=True)
    preflight_parser.add_argument("--sums", type=Path, required=True)
    preflight_parser.add_argument("--tag", required=True)
    preflight_parser.add_argument("--run-revision", required=True)
    preflight_parser.add_argument("--rehearsal-run", required=True)
    preflight_parser.add_argument("--state-out", type=Path, required=True)
    preflight_parser.add_argument("--plan-out", type=Path, required=True)
    preflight_parser.add_argument("--receipt-out", type=Path, required=True)
    preflight_parser.add_argument("--require-credentials", action="store_true")
    preflight_parser.set_defaults(run=preflight)

    publish_parser = commands.add_parser("publish")
    publish_parser.add_argument("--account", type=Path, required=True)
    publish_parser.add_argument("--candidates", type=Path, required=True)
    publish_parser.add_argument("--sums", type=Path, required=True)
    publish_parser.add_argument("--state", type=Path, required=True)
    publish_parser.add_argument("--plan", type=Path, required=True)
    publish_parser.add_argument("--out", type=Path, required=True)
    publish_parser.set_defaults(run=publish)

    image_parser = commands.add_parser("image-state")
    image_parser.add_argument("--account", type=Path, required=True)
    image_parser.add_argument("--candidates", type=Path, required=True)
    image_parser.add_argument("--id", required=True)
    image_parser.set_defaults(run=image_state)

    complete_parser = commands.add_parser("complete")
    complete_parser.add_argument("--account", type=Path, required=True)
    complete_parser.add_argument("--candidates", type=Path, required=True)
    complete_parser.add_argument("--sums", type=Path, required=True)
    complete_parser.add_argument("--rehearsal-run", required=True)
    complete_parser.add_argument("--publication-run", required=True)
    complete_parser.add_argument("--out", type=Path, required=True)
    complete_parser.set_defaults(run=complete)

    qualify_parser = commands.add_parser("qualify")
    qualify_parser.add_argument("--out", type=Path, required=True)
    qualify_parser.set_defaults(run=qualify)
    return root


def main():
    arguments = parser().parse_args()
    arguments.run(arguments)


if __name__ == "__main__":
    try:
        main()
    except (PublicationError, OrchestrationError, subprocess.CalledProcessError) as error:
        raise SystemExit(f"public release failed: {error}") from error
