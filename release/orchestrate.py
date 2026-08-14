#!/usr/bin/env python3

import argparse
import copy
import hashlib
import json
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
CATALOG_FILE = ROOT / "release/artifacts.json"
ORDINARY_WORKFLOW = ROOT / ".github/workflows/ci.yml"
RELEASE_WORKFLOW = ROOT / ".github/workflows/release.yml"
ROOT_GATE = ROOT / "scripts/check.sh"


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


def workflow_account(root=ROOT):
    root = Path(root)
    ordinary = (root / ORDINARY_WORKFLOW.relative_to(ROOT)).read_text()
    gate = (root / ROOT_GATE.relative_to(ROOT)).read_text()
    release = (root / RELEASE_WORKFLOW.relative_to(ROOT)).read_text()
    ordinary_commands = re.findall(r"^\s*-\s+run:\s*(.+?)\s*$", ordinary, re.MULTILINE)
    require(ordinary_commands == ["./scripts/check.sh"], "ordinary CI has a non-canonical command")
    require("--release-images" in gate, "root gate has no explicit release-image entry point")
    require(
        gate.count("qualify.py --images") == 1,
        "root gate does not isolate one release-only image entry point",
    )
    for lane in ("packages", "native", "images", "account"):
        require(
            re.search(rf"^  {lane}:$", release, re.MULTILINE),
            f"release lane {lane!r} is absent",
        )
    require(release.count("fail-fast: false") == 2, "release matrices do not isolate failures")
    require(
        "needs: [packages, native, images]" in release and "always()" in release,
        "release account does not observe every lane outcome",
    )
    require("cache-from: type=gha" in release, "release image lane has no BuildKit cache input")
    require(
        "cache-to: type=gha,mode=max" in release,
        "release image lane has no BuildKit cache output",
    )
    require("actions/attest-build-provenance@v4" in release, "release workflow omits provenance")
    for subject_path in (
        "dist/packages/*",
        "dist/native/${{ matrix.archive }}",
        "dist/images/${{ matrix.archive }}",
    ):
        require(
            f"subject-path: {subject_path}" in release,
            f"release workflow omits provenance subject {subject_path!r}",
        )
    return {
        "ordinaryCommand": ordinary_commands[0],
        "releaseLanes": ["packages", "native", "images", "account"],
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
    matrix = native_matrix(catalog) if arguments.kind == "native" else image_matrix(catalog)
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


def parser():
    root = argparse.ArgumentParser()
    commands = root.add_subparsers(dest="command", required=True)

    matrix = commands.add_parser("matrix")
    matrix.add_argument("kind", choices=("native", "image"))
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
    return root


def main():
    arguments = parser().parse_args()
    arguments.run(arguments)


if __name__ == "__main__":
    try:
        main()
    except (OrchestrationError, subprocess.CalledProcessError) as error:
        raise SystemExit(f"release orchestration failed: {error}") from error
