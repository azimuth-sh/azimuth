#!/usr/bin/env python3

import argparse
import copy
import hashlib
import json
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
EVIDENCE_RUN_URL = re.compile(
    r"https://github\.com/(?:drim-dev|azimuth-sh)/azimuth/actions/runs/[0-9]+"
)
CATALOG_FILE = ROOT / "release/artifacts.json"
ORDINARY_WORKFLOW = ROOT / ".github/workflows/ci.yml"
RELEASE_WORKFLOW = ORDINARY_WORKFLOW
ROOT_GATE = ROOT / "scripts/check.sh"
OUTPUT_ROOT = ROOT / ".azimuth/release"
ORDINARY_RECEIPT = OUTPUT_ROOT / "ordinary-workflow-receipt.json"
RELEASE_RECEIPT = OUTPUT_ROOT / "release-workflow-receipt.json"
SPEC = "framework/release-orchestration"


class OrchestrationError(Exception):
    pass


def require(condition, message):
    if not condition:
        raise OrchestrationError(message)


def read_json(path):
    try:
        return json.loads(Path(path).read_text())
    except (OSError, json.JSONDecodeError) as error:
        raise OrchestrationError(f"cannot read {path}: {error}") from error


def catalog_at(root=ROOT):
    return read_json(Path(root) / "release/artifacts.json")


def npm_archive(identity, version):
    return f"{identity.removeprefix('@').replace('/', '-')}-{version}.tgz"


def package_archive(package, version):
    ecosystem = package["ecosystem"]
    if ecosystem == "cargo":
        return f"{package['identity']}-{version}.crate"
    if ecosystem == "nuget":
        return f"{package['identity']}.{version}.nupkg"
    return npm_archive(package["identity"], version)


def native_archive(identity, target, version):
    extension = "zip" if target.endswith("windows-msvc") else "tar.gz"
    return f"{identity}-{version}-{target}.{extension}"


def native_matrix(catalog):
    runners = {
        "x86_64-unknown-linux-gnu": ("ubuntu-latest", "azimuth"),
        "aarch64-apple-darwin": ("macos-14", "azimuth"),
        "x86_64-pc-windows-msvc": ("windows-latest", "azimuth.exe"),
    }
    version = catalog["release"]["version"]
    identity = catalog["nativeBinaries"]["identity"]
    matrix = []
    for target in catalog["nativeBinaries"]["targets"]:
        require(target in runners, f"native target {target!r} has no selected runner")
        runner, binary = runners[target]
        matrix.append(
            {
                "target": target,
                "runner": runner,
                "binary": binary,
                "archive": native_archive(identity, target, version),
            }
        )
    return matrix


def image_matrix(catalog):
    return [
        {
            "id": image["id"],
            "identity": image["identity"],
            "version": catalog["release"]["version"],
            "context": image["context"],
            "dockerfile": image["dockerfile"],
            "platforms": ",".join(sorted(image["platforms"])),
            "archive": f"{image['id']}.oci.tar",
        }
        for image in catalog["images"]
    ]


def image_platform_matrix(catalog):
    runners = {
        "linux/amd64": ("ubuntu-24.04", "amd64"),
        "linux/arm64": ("ubuntu-24.04-arm", "arm64"),
    }
    matrix = []
    for image in catalog["images"]:
        for platform in sorted(image["platforms"]):
            require(platform in runners, f"image platform {platform!r} has no selected runner")
            runner, architecture = runners[platform]
            matrix.append(
                {
                    "id": image["id"],
                    "identity": image["identity"],
                    "version": catalog["release"]["version"],
                    "context": image["context"],
                    "dockerfile": image["dockerfile"],
                    "platform": platform,
                    "architecture": architecture,
                    "runner": runner,
                    "archive": f"{image['id']}-{architecture}.oci.tar",
                }
            )
    return matrix


def expected_subjects(catalog):
    version = catalog["release"]["version"]
    subjects = []
    for package in catalog["packages"]:
        subjects.append(
            {
                "key": f"package:{package['id']}",
                "kind": "package",
                "id": package["id"],
                "identity": package["identity"],
                "ecosystem": package["ecosystem"],
                "filename": package_archive(package, version),
            }
        )
    for native in native_matrix(catalog):
        subjects.append(
            {
                "key": f"native:{native['target']}",
                "kind": "native",
                "id": native["target"],
                "identity": catalog["nativeBinaries"]["identity"],
                "target": native["target"],
                "filename": native["archive"],
            }
        )
    for image in catalog["images"]:
        subjects.append(
            {
                "key": f"image:{image['id']}",
                "kind": "image",
                "id": image["id"],
                "identity": image["identity"],
                "platforms": sorted(image["platforms"]),
                "filename": f"{image['id']}.oci.tar",
            }
        )
    return subjects


def workflow_jobs(source):
    lines = source.splitlines()
    try:
        start = lines.index("jobs:") + 1
    except ValueError as error:
        raise OrchestrationError("workflow has no jobs mapping") from error
    jobs = {}
    name = None
    body = []
    for line in lines[start:]:
        match = re.fullmatch(r"  ([A-Za-z0-9_-]+):", line)
        if match:
            if name is not None:
                jobs[name] = "\n".join(body)
            name = match.group(1)
            body = [line]
        elif name is not None:
            if line and not line.startswith(" "):
                break
            body.append(line)
    if name is not None:
        jobs[name] = "\n".join(body)
    return jobs


def workflow_run_commands(job):
    lines = job.splitlines()
    commands = []
    index = 0
    while index < len(lines):
        match = re.match(r"^(\s*)-\s+run:\s*(.*?)\s*$", lines[index])
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


def workflow_account(root=ROOT):
    root = Path(root)
    workflow = (root / ORDINARY_WORKFLOW.relative_to(ROOT)).read_text()
    gate = (root / ROOT_GATE.relative_to(ROOT)).read_text()
    jobs = workflow_jobs(workflow)
    require("--release-images" in gate, "root gate has no explicit release-image entry point")
    require(
        gate.count("qualify.py --images") == 1,
        "root gate does not isolate one release-only image entry point",
    )
    lanes = (
        "prepare", "source", "assurance", "packages", "native", "image-platforms",
        "deployment", "images", "release_check", "account", "check",
    )
    for lane in lanes:
        require(lane in jobs, f"candidate lane {lane!r} is absent")
    source_commands = workflow_run_commands(jobs["source"])
    require(
        source_commands == ["./scripts/check.sh source"],
        "candidate source lane has a non-canonical command",
    )
    assurance_commands = workflow_run_commands(jobs["assurance"])
    require(
        assurance_commands == ["./scripts/check.sh assurance"],
        "candidate assurance lane has a non-canonical command",
    )
    for lane in ("native", "image-platforms", "images"):
        fail_fast = re.findall(r"^\s+fail-fast:\s*(\S+)\s*$", jobs[lane], re.MULTILINE)
        require(fail_fast == ["false"], f"release matrix {lane!r} does not isolate failures")
    require(
        "needs: [prepare, image-platforms]" in jobs["images"],
        "release image assembly does not require every native platform build",
    )
    platform_job = jobs["image-platforms"]
    require(
        "runs-on: ${{ matrix.runner }}" in platform_job
        and "platforms: ${{ matrix.platform }}" in platform_job,
        "release image platform build is not bound to its selected native runner",
    )
    require(
        "docker/setup-qemu-action" not in platform_job,
        "release image platform build reintroduces emulation",
    )
    require(
        "scope=candidate-${{ matrix.id }}-${{ matrix.architecture }}" in platform_job,
        "release image cache is not isolated by image and architecture",
    )
    require(
        "needs: [packages, native, images]" in jobs["account"] and "always()" in jobs["account"],
        "release account does not observe every lane outcome",
    )
    require("cache-from: type=gha" in workflow, "release image lane has no BuildKit cache input")
    require(
        "cache-to: type=gha,mode=max" in workflow,
        "release image lane has no BuildKit cache output",
    )
    require("actions/attest-build-provenance@v4" in workflow, "release workflow omits provenance")
    require(
        '[[ "$GITHUB_EVENT_NAME" == "pull_request" ]]' in jobs["account"]
        and 'tag --force --annotate "$tag"' in jobs["account"],
        "release pull request does not isolate its synthetic tag",
    )
    for subject_path in (
        "dist/packages/*",
        "dist/native/${{ matrix.archive }}",
        "dist/images/${{ matrix.archive }}",
    ):
        require(
            f"subject-path: {subject_path}" in workflow,
            f"release workflow omits provenance subject {subject_path!r}",
        )
    deployment = jobs["deployment"]
    require(
        "needs: image-platforms" in deployment
        and "pattern: image-platform-*-amd64" in deployment
        and "sudo apt-get install --yes skopeo" in deployment
        and deployment.count("release/candidates.py import-image") == 2
        and "--prebuilt-images" in deployment
        and "--build" not in deployment,
        "deployment lane does not consume both retained amd64 image fragments",
    )
    release_check = jobs["release_check"]
    require(
        "needs: [source, packages]" in release_check
        and "--candidates dist/packages" in release_check
        and "--defer-hosted-receipts" in release_check,
        "release qualification does not reuse retained packages after source checks",
    )
    require(
        "needs: [source, assurance, deployment, release_check, account]" in jobs["check"],
        "final candidate check does not observe every verification lane",
    )
    return {
        "ordinaryCommand": source_commands[0],
        "releaseLanes": list(lanes),
        "releaseImagesInOrdinaryGate": False,
    }


def verify_tag(root, tag, revision):
    result = subprocess.run(
        ["git", "rev-list", "-n", "1", tag],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    )
    require(result.stdout.strip() == revision, f"tag {tag!r} does not name revision {revision}")


def file_index(candidate_root):
    index = {}
    for path in Path(candidate_root).rglob("*"):
        if not path.is_file():
            continue
        index.setdefault(path.name, []).append(path)
    return index


def digest(path):
    checksum = hashlib.sha256()
    with Path(path).open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            checksum.update(block)
    return checksum.hexdigest()


def combined_digest(paths):
    checksum = hashlib.sha256()
    for path in paths:
        checksum.update(Path(path).read_bytes())
    return checksum.hexdigest()


def git_revision_is_ancestor(root, revision):
    result = subprocess.run(
        ["git", "merge-base", "--is-ancestor", revision, "HEAD"],
        cwd=root,
        capture_output=True,
    )
    return result.returncode == 0


def expected_release_jobs(catalog):
    return [
        "packages",
        *(f"native:{item['target']}" for item in native_matrix(catalog)),
        *(f"image:{item['id']}" for item in image_matrix(catalog)),
        "account",
    ]


def validate_ordinary_receipt(receipt, root=ROOT, ancestor=git_revision_is_ancestor):
    root = Path(root)
    expected = {
        "format": "azimuth-ordinary-workflow-receipt",
        "schemaVersion": 1,
        "workflow": ".github/workflows/ci.yml",
        "conclusion": "success",
        "workflowSha256": digest(root / ".github/workflows/ci.yml"),
        "rootGateSha256": digest(root / "scripts/check.sh"),
    }
    for field, value in expected.items():
        require(receipt.get(field) == value, f"ordinary receipt {field} differs")
    source_revision = receipt.get("sourceRevision", "")
    require(re.fullmatch(r"[0-9a-f]{40}", source_revision) is not None,
            "ordinary receipt sourceRevision is invalid")
    require(ancestor(root, source_revision), "ordinary receipt source revision is not current")
    require(re.fullmatch(r"[0-9a-f]{40}", receipt.get("executionRevision", "")) is not None,
            "ordinary receipt executionRevision is invalid")
    require(EVIDENCE_RUN_URL.fullmatch(receipt.get("runUrl", "")) is not None,
            "ordinary receipt runUrl is invalid")
    duration = receipt.get("durationSeconds")
    require(isinstance(duration, int) and 0 < duration < 45 * 60,
            "ordinary receipt exceeds the hosted-job limit")
    return receipt


def validate_release_receipt(receipt, root=ROOT, ancestor=git_revision_is_ancestor):
    root = Path(root)
    catalog = catalog_at(root)
    expected = {
        "format": "azimuth-release-workflow-receipt",
        "schemaVersion": 1,
        "workflow": ".github/workflows/ci.yml",
        "conclusion": "success",
        "workflowSha256": digest(root / ".github/workflows/ci.yml"),
        "accountSha256": digest(root / "release/orchestrate.py"),
        "consumerSha256": digest(root / "release/candidates.py"),
    }
    for field, value in expected.items():
        require(receipt.get(field) == value, f"release receipt {field} differs")
    source_revision = receipt.get("sourceRevision", "")
    require(re.fullmatch(r"[0-9a-f]{40}", source_revision) is not None,
            "release receipt sourceRevision is invalid")
    require(ancestor(root, source_revision), "release receipt source revision is not current")
    require(re.fullmatch(r"[0-9a-f]{40}", receipt.get("executionRevision", "")) is not None,
            "release receipt executionRevision is invalid")
    require(EVIDENCE_RUN_URL.fullmatch(receipt.get("runUrl", "")) is not None,
            "release receipt runUrl is invalid")
    require(receipt.get("jobs") == expected_release_jobs(catalog),
            "release receipt job population differs")
    expected_names = [item["filename"] for item in expected_subjects(catalog)]
    require(receipt.get("subjects") == expected_names,
            "release receipt subject population differs")
    require(receipt.get("attestedSubjects") == expected_names,
            "release receipt provenance population differs")
    require(re.fullmatch(r"[0-9a-f]{64}", receipt.get("candidateAccountSha256", "")) is not None,
            "release receipt candidate account digest is invalid")
    return receipt


def assemble(candidate_root, revision, tag, root=ROOT):
    require(re.fullmatch(r"[0-9a-f]{40}", revision) is not None, "revision is not a full Git id")
    catalog = catalog_at(root)
    release = catalog["release"]
    require(tag == release["tag"], f"tag {tag!r} differs from catalog tag {release['tag']!r}")
    files = file_index(candidate_root)
    expected = expected_subjects(catalog)
    expected_names = {subject["filename"] for subject in expected}
    unexpected = sorted(set(files) - expected_names)
    require(not unexpected, f"candidate account contains unexpected files {unexpected}")

    subjects = []
    for expected_subject in expected:
        matches = files.get(expected_subject["filename"], [])
        require(matches, f"candidate account omits {expected_subject['filename']}")
        require(len(matches) == 1, f"candidate account duplicates {expected_subject['filename']}")
        path = matches[0]
        subjects.append(
            {
                **expected_subject,
                "size": path.stat().st_size,
                "sha256": digest(path),
            }
        )
    return {
        "format": "azimuth-release-candidate-account",
        "schemaVersion": 1,
        "version": release["version"],
        "tag": tag,
        "revision": revision,
        "subjects": subjects,
    }


def verify(account, candidate_root, root=ROOT):
    catalog = catalog_at(root)
    release = catalog["release"]
    require(account.get("format") == "azimuth-release-candidate-account", "account format differs")
    require(account.get("schemaVersion") == 1, "account schema differs")
    require(account.get("version") == release["version"], "account version differs")
    require(account.get("tag") == release["tag"], "account tag differs")
    require(
        re.fullmatch(r"[0-9a-f]{40}", account.get("revision", "")) is not None,
        "account revision is not a full Git id",
    )
    expected = {subject["key"]: subject for subject in expected_subjects(catalog)}
    observed = account.get("subjects", [])
    require(len(observed) == len(expected), "account subject count differs")
    require(
        len({subject.get("key") for subject in observed}) == len(observed),
        "account duplicates a key",
    )
    files = file_index(candidate_root)
    for subject in observed:
        key = subject.get("key")
        require(key in expected, f"account contains unexpected subject {key!r}")
        for field, value in expected[key].items():
            require(subject.get(field) == value, f"{key}: {field} differs")
        matches = files.get(subject["filename"], [])
        require(len(matches) == 1, f"{key}: retained file population differs")
        path = matches[0]
        require(subject.get("size") == path.stat().st_size, f"{key}: byte size differs")
        require(subject.get("sha256") == digest(path), f"{key}: checksum differs")
    return account


def publication_targets(account):
    return [
        {
            "key": subject["key"],
            "identity": subject["identity"],
            "sha256": subject["sha256"],
            **({"platforms": subject["platforms"]} if "platforms" in subject else {}),
        }
        for subject in account["subjects"]
    ]


def plan_publication(account, registry_state):
    targets = {target["key"]: target for target in publication_targets(account)}
    state = registry_state.get("targets", {})
    require(not (set(state) - set(targets)), "registry state contains an unexpected target")
    conflicts = []
    publish = []
    preserve = []
    for key, target in targets.items():
        observed = state.get(key)
        if observed is None:
            publish.append(key)
            continue
        exact = (
            observed.get("identity") == target["identity"]
            and observed.get("sha256") == target["sha256"]
        )
        if "platforms" in target:
            exact = exact and sorted(observed.get("platforms", [])) == target["platforms"]
        if exact:
            preserve.append(key)
        else:
            conflicts.append(key)
    require(not conflicts, f"immutable registry conflicts exist for {sorted(conflicts)}")
    return {"publish": sorted(publish), "preserve": sorted(preserve)}


def validate_completion(account, registry_state):
    plan = plan_publication(account, registry_state)
    require(not plan["publish"], f"public release omits {plan['publish']}")
    targets = registry_state.get("targets", {})
    missing_provenance = sorted(
        key for key in plan["preserve"] if not targets[key].get("provenance", False)
    )
    require(not missing_provenance, f"public release omits provenance for {missing_provenance}")
    return {"outcome": "complete", "targets": plan["preserve"]}


def exact_registry_state(account):
    return {
        "targets": {
            target["key"]: {
                "identity": target["identity"],
                "sha256": target["sha256"],
                "provenance": True,
                **({"platforms": target["platforms"]} if "platforms" in target else {}),
            }
            for target in publication_targets(account)
        }
    }


def rehearse_publication(account):
    exact = exact_registry_state(account)
    preserved = plan_publication(account, exact)
    require(not preserved["publish"], "exact registry state selected a publication")
    absent = []
    for key in exact["targets"]:
        state = copy.deepcopy(exact)
        del state["targets"][key]
        plan = plan_publication(account, state)
        require(plan["publish"] == [key], f"absent target {key!r} was not isolated")
        absent.append(key)
    conflicts = []
    for kind in ("package", "native", "image"):
        subject = next(item for item in account["subjects"] if item["kind"] == kind)
        state = copy.deepcopy(exact)
        state["targets"][subject["key"]]["sha256"] = "f" * 64
        try:
            plan_publication(account, state)
        except OrchestrationError:
            conflicts.append(subject["key"])
        else:
            raise OrchestrationError(f"{kind} conflict produced a publication plan")
    validate_completion(account, exact)
    return {
        "format": "azimuth-release-publication-rehearsal",
        "schemaVersion": 1,
        "preserved": preserved["preserve"],
        "individuallyAbsent": absent,
        "rejectedConflictKinds": conflicts,
        "completion": "passed",
    }


def source_entry(claim, site, file, fingerprint):
    language = {
        ".json": "github-actions-receipt",
        ".py": "python",
        ".yml": "yaml",
    }.get(Path(file).suffix, "release-orchestration")
    return {
        "spec": SPEC,
        "claim": claim,
        "site": site,
        "file": file,
        "lang": language,
        "source_fingerprint": f"sha256:{fingerprint}",
    }


def write_linkage(root, output_root, ordinary_receipt=None, release_receipt=None):
    root = Path(root)
    orchestrator = root / "release/orchestrate.py"
    candidates = root / "release/candidates.py"
    workflow = root / ".github/workflows/ci.yml"
    orchestrator_fingerprint = digest(orchestrator)
    candidates_fingerprint = digest(candidates)
    workflow_fingerprint = digest(workflow)
    realization_sites = [
        ("qualification-lanes-converge", "workflow_account", "release/orchestrate.py", orchestrator_fingerprint),
        ("qualification-lanes-converge", "release_rehearsal_dag", ".github/workflows/ci.yml", workflow_fingerprint),
        ("qualification-lanes-converge", "assemble", "release/orchestrate.py", orchestrator_fingerprint),
        ("tagged-candidates-are-verifiable", "verify_tag", "release/orchestrate.py", orchestrator_fingerprint),
        ("tagged-candidates-are-verifiable", "verify", "release/orchestrate.py", orchestrator_fingerprint),
        ("tagged-candidates-are-verifiable", "release_provenance", ".github/workflows/ci.yml", workflow_fingerprint),
        ("qualified-candidates-compose", "smoke_packages", "release/candidates.py", candidates_fingerprint),
        ("qualified-candidates-compose", "build_native", "release/candidates.py", candidates_fingerprint),
        ("qualified-candidates-compose", "smoke_image", "release/candidates.py", candidates_fingerprint),
        ("partial-publication-resumes-safely", "plan_publication", "release/orchestrate.py", orchestrator_fingerprint),
        ("partial-publication-resumes-safely", "validate_completion", "release/orchestrate.py", orchestrator_fingerprint),
    ]
    linkage = {
        "realizes": [
            source_entry(claim, site, file, fingerprint)
            for claim, site, file, fingerprint in realization_sites
        ],
        "check_implementations": [],
        "mechanism_implementations": [],
        "class_members": [],
        "enumerations": [],
        "artifacts": [
            {
                "id": "release-rehearsal-account",
                "kind": "hosted-workflow-dag",
                "file": ".github/workflows/ci.yml",
            },
            {
                "id": "release-candidate-manifest",
                "kind": "candidate-account-guard",
                "file": "release/orchestrate.py",
            },
            {
                "id": "release-consumer-rehearsal",
                "kind": "disposable-consumer-guard",
                "file": "release/candidates.py",
            },
            {
                "id": "release-publication-plan",
                "kind": "registry-state-guard",
                "file": "release/orchestrate.py",
            },
        ],
    }
    (output_root / "orchestration-linkage.json").write_text(
        json.dumps(linkage, indent=2) + "\n"
    )


def qualify_orchestration(root=ROOT, output_root=OUTPUT_ROOT, validate_receipts=True):
    root = Path(root)
    output_root = Path(output_root)
    account = workflow_account(root)
    ordinary_path = output_root / ORDINARY_RECEIPT.name
    release_path = output_root / RELEASE_RECEIPT.name
    ordinary_receipt = (
        validate_ordinary_receipt(read_json(ordinary_path), root)
        if validate_receipts and ordinary_path.is_file()
        else None
    )
    release_receipt = (
        validate_release_receipt(read_json(release_path), root)
        if validate_receipts and release_path.is_file()
        else None
    )
    catalog = catalog_at(root)
    qualification = {
        "format": "azimuth-release-orchestration-qualification",
        "schemaVersion": 1,
        "workflow": account,
        "nativeMatrix": native_matrix(catalog),
        "imagePlatformMatrix": image_platform_matrix(catalog),
        "imageMatrix": image_matrix(catalog),
        "subjects": expected_subjects(catalog),
        "ordinaryExecution": ordinary_receipt or {"status": "pending"},
        "releaseExecution": release_receipt or {"status": "pending"},
    }
    output_root.mkdir(parents=True, exist_ok=True)
    (output_root / "release-orchestration.json").write_text(
        json.dumps(qualification, indent=2) + "\n"
    )
    write_linkage(root, output_root, ordinary_receipt, release_receipt)
    return qualification


def write_account(account, output):
    output = Path(output)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(account, indent=2) + "\n")
    checksums = output.with_name("SHA256SUMS")
    checksums.write_text(
        "".join(f"{subject['sha256']}  {subject['filename']}\n" for subject in account["subjects"])
    )


def command_matrix(arguments):
    catalog = catalog_at(arguments.root)
    if arguments.kind == "native":
        matrix = native_matrix(catalog)
    elif arguments.kind == "image-platform":
        matrix = image_platform_matrix(catalog)
    else:
        matrix = image_matrix(catalog)
    print(json.dumps({"include": matrix}, separators=(",", ":")))


def command_assemble(arguments):
    if arguments.verify_tag:
        verify_tag(arguments.root, arguments.tag, arguments.revision)
    account = assemble(arguments.candidates, arguments.revision, arguments.tag, arguments.root)
    write_account(account, arguments.out)
    print(f"assembled {len(account['subjects'])} retained release candidate(s)")


def command_verify(arguments):
    account = verify(read_json(arguments.account), arguments.candidates, arguments.root)
    print(f"verified {len(account['subjects'])} retained release candidate(s)")


def command_plan(arguments):
    plan = plan_publication(read_json(arguments.account), read_json(arguments.state))
    Path(arguments.out).write_text(json.dumps(plan, indent=2) + "\n")
    print(f"publication plan: publish={len(plan['publish'])} preserve={len(plan['preserve'])}")


def command_complete(arguments):
    result = validate_completion(read_json(arguments.account), read_json(arguments.state))
    print(f"public release complete: targets={len(result['targets'])}")


def command_rehearse(arguments):
    result = rehearse_publication(read_json(arguments.account))
    Path(arguments.out).write_text(json.dumps(result, indent=2) + "\n")
    print(f"rehearsed {len(result['individuallyAbsent'])} resumable target(s)")


def command_workflows(arguments):
    account = workflow_account(arguments.root)
    print(json.dumps(account, indent=2))


def command_qualify(arguments):
    result = qualify_orchestration(
        arguments.root,
        arguments.out,
        validate_receipts=not arguments.defer_hosted_receipts,
    )
    print(
        f"qualified {len(result['subjects'])} retained subject(s), "
        f"{len(result['nativeMatrix'])} native runner(s), and "
        f"{len(result['imagePlatformMatrix'])} native image build(s) across "
        f"{len(result['imageMatrix'])} image lane(s)"
    )


def parser():
    root = argparse.ArgumentParser()
    commands = root.add_subparsers(dest="command", required=True)

    matrix = commands.add_parser("matrix")
    matrix.add_argument("kind", choices=("native", "image-platform", "image"))
    matrix.add_argument("--root", type=Path, default=ROOT)
    matrix.set_defaults(run=command_matrix)

    assemble_parser = commands.add_parser("assemble")
    assemble_parser.add_argument("--root", type=Path, default=ROOT)
    assemble_parser.add_argument("--candidates", type=Path, required=True)
    assemble_parser.add_argument("--revision", required=True)
    assemble_parser.add_argument("--tag", required=True)
    assemble_parser.add_argument("--out", type=Path, required=True)
    assemble_parser.add_argument("--verify-tag", action="store_true")
    assemble_parser.set_defaults(run=command_assemble)

    verify_parser = commands.add_parser("verify")
    verify_parser.add_argument("--root", type=Path, default=ROOT)
    verify_parser.add_argument("--candidates", type=Path, required=True)
    verify_parser.add_argument("--account", type=Path, required=True)
    verify_parser.set_defaults(run=command_verify)

    plan = commands.add_parser("plan")
    plan.add_argument("--account", type=Path, required=True)
    plan.add_argument("--state", type=Path, required=True)
    plan.add_argument("--out", type=Path, required=True)
    plan.set_defaults(run=command_plan)

    complete = commands.add_parser("complete")
    complete.add_argument("--account", type=Path, required=True)
    complete.add_argument("--state", type=Path, required=True)
    complete.set_defaults(run=command_complete)

    rehearse = commands.add_parser("rehearse")
    rehearse.add_argument("--account", type=Path, required=True)
    rehearse.add_argument("--out", type=Path, required=True)
    rehearse.set_defaults(run=command_rehearse)

    workflows = commands.add_parser("workflows")
    workflows.add_argument("--root", type=Path, default=ROOT)
    workflows.set_defaults(run=command_workflows)

    qualify = commands.add_parser("qualify")
    qualify.add_argument("--root", type=Path, default=ROOT)
    qualify.add_argument("--out", type=Path, default=OUTPUT_ROOT)
    qualify.add_argument("--defer-hosted-receipts", action="store_true")
    qualify.set_defaults(run=command_qualify)
    return root


def main():
    arguments = parser().parse_args()
    arguments.run(arguments)


if __name__ == "__main__":
    try:
        main()
    except (OrchestrationError, subprocess.CalledProcessError) as error:
        raise SystemExit(f"release orchestration failed: {error}") from error
