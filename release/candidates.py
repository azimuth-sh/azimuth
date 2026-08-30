#!/usr/bin/env python3

import argparse
import hashlib
import io
import json
import os
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import time
import urllib.request
import zipfile
from pathlib import Path

if __package__:
    from .qualify import (
        QualificationError,
        archive_files,
        archive_metadata,
        cargo_candidate,
        catalog_at,
        npm_candidate,
        nuget_candidate,
        validate_approved_contract,
        validate_archive_metadata,
        validate_catalog,
        validate_file_set,
        validate_source_metadata,
    )
else:
    from qualify import (
        QualificationError,
        archive_files,
        archive_metadata,
        cargo_candidate,
        catalog_at,
        npm_candidate,
        nuget_candidate,
        validate_approved_contract,
        validate_archive_metadata,
        validate_catalog,
        validate_file_set,
        validate_source_metadata,
    )


ROOT = Path(__file__).resolve().parent.parent


class CandidateError(Exception):
    pass


OCI_INDEX_MEDIA_TYPE = "application/vnd.oci.image.index.v1+json"
OCI_BLOB = re.compile(r"blobs/sha256/([0-9a-f]{64})")


def require(condition, message):
    if not condition:
        raise CandidateError(message)


def run(command, cwd=None, capture=False, check=True):
    return subprocess.run(
        [str(item) for item in command],
        cwd=cwd,
        check=check,
        capture_output=capture,
        text=capture,
    )


def retained_path(candidate, output):
    destination = output / candidate.name
    if candidate.resolve() != destination.resolve():
        shutil.copy2(candidate, destination)
    return destination


def build_packages(root, output, allow_dirty):
    root = Path(root).resolve()
    output = Path(output).resolve()
    catalog = catalog_at(root)
    validate_catalog(catalog, root)
    validate_approved_contract(catalog)
    validate_source_metadata(catalog, root)
    output.mkdir(parents=True, exist_ok=True)
    retained = {}
    for package in catalog["packages"]:
        if package["ecosystem"] == "cargo":
            candidate = cargo_candidate(package, root, allow_dirty)
        elif package["ecosystem"] == "nuget":
            candidate = nuget_candidate(package, root, output)
        else:
            package_root = (root / package["manifest"]).parent
            run(["npm", "ci", "--ignore-scripts"], cwd=package_root)
            candidate = npm_candidate(package, root, output)
        candidate = retained_path(candidate, output)
        files = archive_files(candidate, package["ecosystem"])
        validate_file_set(package, files)
        validate_archive_metadata(
            package,
            archive_metadata(candidate, package["ecosystem"]),
            catalog["release"],
        )
        retained[package["id"]] = candidate
    smoke_packages(catalog, retained)
    return retained


def smoke_cargo(catalog, archive):
    with tempfile.TemporaryDirectory(prefix="azimuth-cargo-consumer-") as temporary:
        consumer = Path(temporary)
        with tarfile.open(archive, "r:gz") as package:
            package.extractall(consumer, filter="data")
        source = next(path for path in consumer.iterdir() if path.is_dir())
        install = consumer / "install"
        run(["cargo", "install", "--locked", "--path", source, "--root", install])
        result = run([install / "bin/azimuth", "--version"], capture=True)
        require(catalog["release"]["version"] in result.stdout, "Cargo CLI version differs")


def smoke_dotnet(catalog, annotations, emitter):
    version = catalog["release"]["version"]
    with tempfile.TemporaryDirectory(prefix="azimuth-dotnet-consumer-") as temporary:
        consumer = Path(temporary)
        project = consumer / "AnnotationsConsumer.csproj"
        project.write_text(
            "<Project Sdk=\"Microsoft.NET.Sdk\">\n"
            "  <PropertyGroup><OutputType>Exe</OutputType><TargetFramework>net10.0"
            "</TargetFramework></PropertyGroup>\n"
            "  <ItemGroup><PackageReference Include=\"Azimuth.Annotations\" "
            f"Version=\"{version}\" /></ItemGroup>\n"
            "</Project>\n"
        )
        (consumer / "Program.cs").write_text(
            "using System;\n"
            "using Azimuth.Annotations;\n"
            "var tag = new RealizesAttribute(\"consumer\", \"starts\");\n"
            "Console.WriteLine($\"{tag.Spec}#{tag.Claim}\");\n"
        )
        config = consumer / "NuGet.Config"
        config.write_text(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n"
            "<configuration><packageSources><clear />"
            f"<add key=\"candidate\" value=\"{annotations.parent}\" />"
            "</packageSources></configuration>\n"
        )
        run(["dotnet", "restore", project, "--configfile", config], cwd=consumer)
        result = run(["dotnet", "run", "--project", project, "--no-restore"],
                     cwd=consumer, capture=True)
        require("consumer#starts" in result.stdout, ".NET annotation entry point failed")

        tools = consumer / "tools"
        run(
            [
                "dotnet",
                "tool",
                "install",
                "--tool-path",
                tools,
                "--add-source",
                emitter.parent,
                "--ignore-failed-sources",
                "Azimuth.Emit",
                "--version",
                version,
            ]
        )
        executable_name = "azimuth-emit-dotnet.exe" if os.name == "nt" else "azimuth-emit-dotnet"
        executable = tools / executable_name
        usage = run([executable], capture=True, check=False)
        require(usage.returncode == 2, ".NET emitter accepted an incomplete invocation")
        require("usage: azimuth-emit-dotnet" in usage.stderr, ".NET emitter entry point failed")


def smoke_typescript(annotations, emitter):
    with tempfile.TemporaryDirectory(prefix="azimuth-npm-consumer-") as temporary:
        consumer = Path(temporary)
        # The emitter derives an ecosystem-semantic site from the package name, so the disposable
        # consumer needs a real one rather than an anonymous private manifest.
        (consumer / "package.json").write_text(
            '{"name":"azimuth-release-smoke-consumer","version":"0.0.0",'
            '"private":true,"type":"commonjs"}\n'
        )
        run(
            [
                "npm",
                "install",
                "--ignore-scripts",
                "--no-audit",
                "--no-fund",
                annotations,
                emitter,
            ],
            cwd=consumer,
        )
        annotation_result = run(
            [
                "node",
                "-e",
                "const a=require('@azimuth-sh/annotations');"
                "a.realizes('consumer','starts');"
                "console.log(typeof a.realizes,typeof a.implementsCheck,typeof a['co'+'vers'])",
            ],
            cwd=consumer,
            capture=True,
        )
        require(
            annotation_result.stdout.strip() == "function function undefined",
            "npm annotation entry point failed",
        )
        source = consumer / "sample.ts"
        source.write_text(
            "import { realizes } from '@azimuth-sh/annotations';\n"
            "export function start(): void { realizes('consumer', 'starts'); }\n"
        )
        # The emitter resolves a site through the nearest TypeScript project, so a consumer
        # without one fails closed rather than guessing an identity from the file path.
        (consumer / "tsconfig.json").write_text(
            json.dumps(
                {
                    "compilerOptions": {
                        "module": "commonjs",
                        "moduleResolution": "node",
                        "noEmit": True,
                        "skipLibCheck": True,
                        "target": "ES2022",
                    },
                    "include": ["sample.ts"],
                },
                indent=2,
            )
            + "\n"
        )
        manifest = consumer / "manifest.json"
        run(
            [
                "node",
                consumer / "node_modules/@azimuth-sh/emit/dist/cli.js",
                "--output",
                manifest,
                "--root",
                consumer,
                source,
            ],
            cwd=consumer,
        )
        emitted = json.loads(manifest.read_text())
        require(
            any(item["spec"] == "consumer" and item["claim"] == "starts"
                for item in emitted["realizes"]),
            "npm emitter entry point failed",
        )


def smoke_packages(catalog, retained):
    smoke_cargo(catalog, retained["rust-cli-core"])
    smoke_dotnet(
        catalog,
        retained["dotnet-annotations"],
        retained["dotnet-emitter"],
    )
    smoke_typescript(
        retained["typescript-annotations"],
        retained["typescript-emitter"],
    )


def native_binary(root, target):
    suffix = ".exe" if target.endswith("windows-msvc") else ""
    return root / "tools/azimuth/target" / target / "release" / f"azimuth{suffix}"


def build_native(root, output, target):
    root = Path(root).resolve()
    output = Path(output).resolve()
    catalog = catalog_at(root)
    require(target in catalog["nativeBinaries"]["targets"], f"native target {target!r} is absent")
    version = catalog["release"]["version"]
    identity = catalog["nativeBinaries"]["identity"]
    run(
        [
            "cargo",
            "build",
            "--locked",
            "--release",
            "--manifest-path",
            "tools/azimuth/Cargo.toml",
            "--target",
            target,
        ],
        cwd=root,
    )
    binary = native_binary(root, target)
    require(binary.is_file(), f"native build omitted {binary.name}")
    output.mkdir(parents=True, exist_ok=True)
    extension = "zip" if target.endswith("windows-msvc") else "tar.gz"
    archive = output / f"{identity}-{version}-{target}.{extension}"
    if extension == "zip":
        with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED) as candidate:
            candidate.write(binary, binary.name)
    else:
        with tarfile.open(archive, "w:gz") as candidate:
            candidate.add(binary, arcname=binary.name)
    with tempfile.TemporaryDirectory(prefix="azimuth-native-consumer-") as temporary:
        consumer = Path(temporary)
        if extension == "zip":
            with zipfile.ZipFile(archive) as candidate:
                candidate.extractall(consumer)
        else:
            with tarfile.open(archive, "r:gz") as candidate:
                candidate.extractall(consumer, filter="data")
        result = run([consumer / binary.name, "--version"], capture=True)
        require(version in result.stdout, f"{target}: retained CLI version differs")
    return archive


def image_contract(root, image_id):
    catalog = catalog_at(root)
    return next((item for item in catalog["images"] if item["id"] == image_id), None)


def read_archive_json(candidate, members, name, archive):
    member = members.get(name)
    require(member is not None, f"{archive.name}: OCI archive omits {name}")
    content = candidate.extractfile(member)
    require(content is not None, f"{archive.name}: cannot read {name}")
    try:
        return json.load(content)
    except json.JSONDecodeError as error:
        raise CandidateError(f"{archive.name}: {name} is malformed JSON: {error}") from error


def copy_oci_blobs(archive, blob_root):
    archive = Path(archive)
    with tarfile.open(archive) as candidate:
        members = {}
        for member in candidate.getmembers():
            if member.isdir():
                require(
                    member.name.rstrip("/") in ("blobs", "blobs/sha256"),
                    f"{archive.name}: unexpected OCI directory {member.name!r}",
                )
                continue
            require(member.isfile(), f"{archive.name}: unexpected OCI member {member.name!r}")
            require(
                member.name in ("index.json", "oci-layout") or OCI_BLOB.fullmatch(member.name),
                f"{archive.name}: unexpected OCI member {member.name!r}",
            )
            require(member.name not in members, f"{archive.name}: duplicate OCI member {member.name!r}")
            members[member.name] = member

        layout = read_archive_json(candidate, members, "oci-layout", archive)
        require(
            layout == {"imageLayoutVersion": "1.0.0"},
            f"{archive.name}: unsupported OCI layout",
        )
        index = read_archive_json(candidate, members, "index.json", archive)

        for name, member in sorted(members.items()):
            match = OCI_BLOB.fullmatch(name)
            if match is None:
                continue
            expected_digest = match.group(1)
            source = candidate.extractfile(member)
            require(source is not None, f"{archive.name}: cannot read {name}")
            destination = blob_root / expected_digest
            checksum = hashlib.sha256()
            temporary = blob_root / f".{expected_digest}.incoming"
            sink = None if destination.is_file() else temporary.open("wb")
            try:
                for block in iter(lambda: source.read(1024 * 1024), b""):
                    checksum.update(block)
                    if sink is not None:
                        sink.write(block)
            finally:
                if sink is not None:
                    sink.close()
            require(
                checksum.hexdigest() == expected_digest,
                f"{archive.name}: blob {expected_digest} checksum differs",
            )
            if destination.is_file():
                require(
                    destination.stat().st_size == member.size,
                    f"{archive.name}: duplicate blob {expected_digest} size differs",
                )
            else:
                temporary.replace(destination)
    return index


def descriptor_content(descriptor, blob_root, archive):
    require(isinstance(descriptor, dict), f"{archive.name}: OCI descriptor is not an object")
    digest = descriptor.get("digest", "")
    require(
        re.fullmatch(r"sha256:[0-9a-f]{64}", digest) is not None,
        f"{archive.name}: OCI descriptor digest is invalid",
    )
    content = blob_root / digest.removeprefix("sha256:")
    require(content.is_file(), f"{archive.name}: OCI descriptor blob {digest} is absent")
    require(
        descriptor.get("size") == content.stat().st_size,
        f"{archive.name}: OCI descriptor {digest} size differs",
    )
    return content


def platform_manifests(archive, blob_root):
    index = copy_oci_blobs(archive, blob_root)
    require(isinstance(index, dict), f"{archive.name}: OCI index is not an object")
    require(index.get("schemaVersion") == 2, f"{archive.name}: OCI index schema differs")
    require(index.get("mediaType") == OCI_INDEX_MEDIA_TYPE, f"{archive.name}: OCI index type differs")
    roots = index.get("manifests", [])
    require(len(roots) == 1, f"{archive.name}: OCI archive must contain one tagged index")
    root = roots[0]
    require(root.get("mediaType") == OCI_INDEX_MEDIA_TYPE, f"{archive.name}: tagged root is not an index")
    root_content = descriptor_content(root, blob_root, archive)
    try:
        selected = json.loads(root_content.read_text())
    except json.JSONDecodeError as error:
        raise CandidateError(f"{archive.name}: tagged index is malformed JSON: {error}") from error
    require(isinstance(selected, dict), f"{archive.name}: tagged index is not an object")
    require(selected.get("schemaVersion") == 2, f"{archive.name}: tagged index schema differs")
    require(
        selected.get("mediaType") == OCI_INDEX_MEDIA_TYPE,
        f"{archive.name}: tagged index type differs",
    )
    manifests = selected.get("manifests", [])
    require(isinstance(manifests, list) and manifests, f"{archive.name}: tagged index is empty")
    for descriptor in manifests:
        descriptor_content(descriptor, blob_root, archive)
    return manifests


def platform_archive_manifest(archive, blob_root):
    manifests = platform_manifests(archive, blob_root)
    platform_descriptors = []
    unknown_descriptors = []
    for descriptor in manifests:
        require(
            isinstance(descriptor, dict),
            f"{archive.name}: OCI descriptor is not an object",
        )
        platform = descriptor.get("platform", {})
        require(
            isinstance(platform, dict),
            f"{archive.name}: OCI descriptor platform is not an object",
        )
        operating_system = platform.get("os")
        architecture = platform.get("architecture")
        if (operating_system, architecture) == ("unknown", "unknown"):
            unknown_descriptors.append(descriptor)
        else:
            platform_descriptors.append(descriptor)
    require(
        len(platform_descriptors) == 1,
        f"{archive.name}: expected exactly one concrete platform manifest",
    )
    descriptor = platform_descriptors[0]
    platform = descriptor["platform"]
    identity = f"{platform.get('os')}/{platform.get('architecture')}"
    for attestation in unknown_descriptors:
        annotations = attestation.get("annotations", {})
        require(
            isinstance(annotations, dict),
            f"{archive.name}: attestation annotations are not an object",
        )
        reference = annotations.get("vnd.docker.reference.digest")
        require(
            reference == descriptor.get("digest"),
            f"{archive.name}: attestation does not name its platform manifest",
        )
    return identity, descriptor, unknown_descriptors


def add_tar_directory(candidate, name):
    member = tarfile.TarInfo(name)
    member.type = tarfile.DIRTYPE
    member.mode = 0o755
    member.mtime = 0
    candidate.addfile(member)


def add_tar_bytes(candidate, name, content):
    member = tarfile.TarInfo(name)
    member.size = len(content)
    member.mode = 0o644
    member.mtime = 0
    candidate.addfile(member, io.BytesIO(content))


def write_oci_archive(output, blob_root, index):
    output = Path(output)
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(prefix=f".{output.name}.", dir=output.parent, delete=False) as staged:
        staged_path = Path(staged.name)
    try:
        with tarfile.open(staged_path, "w") as candidate:
            add_tar_directory(candidate, "blobs/")
            add_tar_directory(candidate, "blobs/sha256/")
            for blob in sorted(blob_root.iterdir(), key=lambda item: item.name):
                require(not blob.name.startswith("."), "incomplete OCI blob remained during assembly")
                member = tarfile.TarInfo(f"blobs/sha256/{blob.name}")
                member.size = blob.stat().st_size
                member.mode = 0o644
                member.mtime = 0
                with blob.open("rb") as content:
                    candidate.addfile(member, content)
            add_tar_bytes(candidate, "index.json", index)
            add_tar_bytes(candidate, "oci-layout", b'{"imageLayoutVersion":"1.0.0"}')
        staged_path.replace(output)
    finally:
        staged_path.unlink(missing_ok=True)


def assemble_image(root, image_id, platform_root, output):
    image = image_contract(root, image_id)
    require(image is not None, f"image {image_id!r} is absent")
    expected_platforms = sorted(image["platforms"])
    archives = sorted(Path(platform_root).glob("*.oci.tar"))
    require(
        len(archives) == len(expected_platforms),
        f"{image_id}: platform archive population differs",
    )

    with tempfile.TemporaryDirectory(prefix=f"azimuth-{image_id}-oci-") as temporary:
        blob_root = Path(temporary) / "blobs"
        blob_root.mkdir()
        selected = {}
        attestations = {}
        for archive in archives:
            identity, descriptor, unknown_descriptors = platform_archive_manifest(
                archive, blob_root
            )
            require(
                identity in expected_platforms,
                f"{archive.name}: unexpected platform {identity!r}",
            )
            require(
                identity not in selected,
                f"{image_id}: duplicate platform {identity!r}",
            )
            selected[identity] = descriptor
            attestations[identity] = unknown_descriptors

        require(
            sorted(selected) == expected_platforms,
            f"{image_id}: assembled platform account differs",
        )

        merged_manifests = [selected[platform] for platform in expected_platforms]
        for platform in expected_platforms:
            merged_manifests.extend(attestations[platform])
        merged = {
            "schemaVersion": 2,
            "mediaType": OCI_INDEX_MEDIA_TYPE,
            "manifests": merged_manifests,
        }
        merged_content = (json.dumps(merged, indent=2) + "\n").encode()
        merged_digest = hashlib.sha256(merged_content).hexdigest()
        (blob_root / merged_digest).write_bytes(merged_content)

        catalog = catalog_at(root)
        version = catalog["release"]["version"]
        tagged = {
            "schemaVersion": 2,
            "mediaType": OCI_INDEX_MEDIA_TYPE,
            "manifests": [
                {
                    "mediaType": OCI_INDEX_MEDIA_TYPE,
                    "digest": f"sha256:{merged_digest}",
                    "size": len(merged_content),
                    "annotations": {
                        "io.containerd.image.name": f"{image['identity']}:{version}",
                        "org.opencontainers.image.ref.name": version,
                    },
                }
            ],
        }
        index_content = json.dumps(tagged, separators=(",", ":")).encode()
        write_oci_archive(output, blob_root, index_content)
    inspect_image(root, image_id, output)
    return Path(output)


def inspect_image(root, image_id, archive):
    if str(root) not in sys.path:
        sys.path.insert(0, str(root))
    from services.assurance.deployment.qualify import inspect_oci_platforms

    image = image_contract(root, image_id)
    require(image is not None, f"image {image_id!r} is absent")
    observed = inspect_oci_platforms(archive)
    require(observed == sorted(image["platforms"]), f"{image_id}: OCI platform account differs")


def published_port(container, target):
    result = run(["docker", "port", container, f"{target}/tcp"], capture=True)
    ports = [line.rsplit(":", 1)[-1] for line in result.stdout.splitlines() if line]
    require(len(ports) == 1 and ports[0].isdigit(), f"cannot resolve port for {container}")
    return int(ports[0])


def wait_for_url(url):
    last_error = None
    for _ in range(60):
        try:
            with urllib.request.urlopen(url, timeout=2) as response:
                require(response.status == 200, f"image entry point returned {response.status}")
                return
        except Exception as error:
            last_error = error
            time.sleep(1)
    raise CandidateError(f"image entry point did not become ready: {last_error}")


def imported_tag(image_id, platform):
    return f"azimuth-rehearsal/{image_id}:{platform.rsplit('/', 1)[-1]}"


def import_image(root, image_id, archive, platform, tag):
    image = image_contract(root, image_id)
    require(image is not None, f"image {image_id!r} is absent")
    require(platform in image["platforms"], f"{image_id}: unexpected platform {platform!r}")
    archive = Path(archive)
    with tempfile.TemporaryDirectory(prefix=f"azimuth-{image_id}-import-") as temporary:
        blob_root = Path(temporary) / "blobs"
        blob_root.mkdir()
        observed, _, _ = platform_archive_manifest(archive, blob_root)
    require(
        observed == platform,
        f"{archive.name}: platform is {observed!r}, expected {platform!r}",
    )
    operating_system, architecture = platform.split("/", 1)
    run(
        [
            "skopeo", "copy", "--override-os", operating_system,
            "--override-arch", architecture, f"oci-archive:{archive}",
            f"docker-daemon:{tag}",
        ]
    )
    inspected = run(
        ["docker", "image", "inspect", tag, "--format", "{{.Os}}/{{.Architecture}}"],
        capture=True,
    )
    require(inspected.stdout.strip() == platform, f"{tag}: imported platform differs")


def smoke_api(tag, platform, suffix):
    network = f"azimuth-release-{suffix}"
    database = f"azimuth-release-db-{suffix}"
    candidate = f"azimuth-release-api-{suffix}"
    run(["docker", "network", "create", network])
    try:
        run(
            [
                "docker", "run", "--detach", "--name", database, "--network", network,
                "--env", "POSTGRES_DB=azimuth", "--env", "POSTGRES_USER=azimuth",
                "--env", "POSTGRES_PASSWORD=rehearsal", "postgres:17-alpine",
            ]
        )
        for _ in range(60):
            ready = run(
                ["docker", "exec", database, "pg_isready", "-U", "azimuth", "-d", "azimuth"],
                check=False,
            )
            if ready.returncode == 0:
                break
            time.sleep(1)
        else:
            raise CandidateError("rehearsal PostgreSQL did not become ready")
        run(
            [
                "docker", "run", "--detach", "--platform", platform, "--name", candidate,
                "--network", network, "--publish", "127.0.0.1::8080", "--env",
                "DATABASE_URL=postgres://azimuth:rehearsal@" + database + ":5432/azimuth", tag,
            ]
        )
        port = published_port(candidate, 8080)
        wait_for_url(f"http://127.0.0.1:{port}/health")
    finally:
        run(["docker", "rm", "--force", candidate, database], check=False)
        run(["docker", "network", "rm", network], check=False)


def smoke_web(tag, platform, suffix):
    candidate = f"azimuth-release-web-{suffix}"
    try:
        run(
            [
                "docker", "run", "--detach", "--platform", platform, "--name", candidate,
                "--publish", "127.0.0.1::3000", "--env",
                "ASSURANCE_API_URL=http://127.0.0.1:9", tag,
            ]
        )
        port = published_port(candidate, 3000)
        wait_for_url(f"http://127.0.0.1:{port}")
    finally:
        run(["docker", "rm", "--force", candidate], check=False)


def smoke_image(root, image_id, archive):
    image = image_contract(root, image_id)
    require(image is not None, f"image {image_id!r} is absent")
    inspect_image(root, image_id, archive)
    for platform in sorted(image["platforms"]):
        architecture = platform.rsplit("/", 1)[-1]
        tag = imported_tag(image_id, platform)
        run(
            [
                "skopeo", "copy", "--override-os", "linux", "--override-arch", architecture,
                f"oci-archive:{archive}", f"docker-daemon:{tag}",
            ]
        )
        suffix = f"{os.getpid()}-{architecture}"
        if image_id == "assurance-api":
            smoke_api(tag, platform, suffix)
        else:
            smoke_web(tag, platform, suffix)
        run(["docker", "image", "rm", tag], check=False)


def parser():
    root = argparse.ArgumentParser()
    commands = root.add_subparsers(dest="command", required=True)

    packages = commands.add_parser("packages")
    packages.add_argument("--root", type=Path, default=ROOT)
    packages.add_argument("--out", type=Path, required=True)
    packages.add_argument("--allow-dirty", action="store_true")

    native = commands.add_parser("native")
    native.add_argument("--root", type=Path, default=ROOT)
    native.add_argument("--out", type=Path, required=True)
    native.add_argument("--target", required=True)

    assemble = commands.add_parser("assemble-image")
    assemble.add_argument("--root", type=Path, default=ROOT)
    assemble.add_argument("--id", required=True)
    assemble.add_argument("--platforms", type=Path, required=True)
    assemble.add_argument("--out", type=Path, required=True)

    inspect = commands.add_parser("inspect-image")
    inspect.add_argument("--root", type=Path, default=ROOT)
    inspect.add_argument("--id", required=True)
    inspect.add_argument("--archive", type=Path, required=True)

    import_parser = commands.add_parser("import-image")
    import_parser.add_argument("--root", type=Path, default=ROOT)
    import_parser.add_argument("--id", required=True)
    import_parser.add_argument("--archive", type=Path, required=True)
    import_parser.add_argument("--platform", required=True)
    import_parser.add_argument("--tag", required=True)

    smoke = commands.add_parser("smoke-image")
    smoke.add_argument("--root", type=Path, default=ROOT)
    smoke.add_argument("--id", required=True)
    smoke.add_argument("--archive", type=Path, required=True)
    return root


def main():
    arguments = parser().parse_args()
    if arguments.command == "packages":
        retained = build_packages(arguments.root, arguments.out, arguments.allow_dirty)
        print(f"retained and exercised {len(retained)} package candidate(s)")
    elif arguments.command == "native":
        archive = build_native(arguments.root, arguments.out, arguments.target)
        print(f"retained and exercised {archive.name}")
    elif arguments.command == "assemble-image":
        archive = assemble_image(arguments.root, arguments.id, arguments.platforms, arguments.out)
        print(f"assembled every selected platform in {archive.name}")
    elif arguments.command == "inspect-image":
        inspect_image(arguments.root, arguments.id, arguments.archive)
        print(f"verified selected platforms in {arguments.archive.name}")
    elif arguments.command == "import-image":
        import_image(
            arguments.root,
            arguments.id,
            arguments.archive,
            arguments.platform,
            arguments.tag,
        )
        print(f"imported {arguments.id} {arguments.platform} as {arguments.tag}")
    else:
        smoke_image(arguments.root, arguments.id, arguments.archive)
        print(f"exercised every selected platform in {arguments.archive.name}")


if __name__ == "__main__":
    try:
        main()
    except (
        CandidateError,
        QualificationError,
        OSError,
        subprocess.CalledProcessError,
        tarfile.TarError,
    ) as error:
        raise SystemExit(f"release candidate failed: {error}") from error
