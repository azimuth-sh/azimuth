#!/usr/bin/env python3

import argparse
import json
import os
import shutil
import socket
import subprocess
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
            package.extractall(consumer)
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
            "Console.WriteLine($\"{tag.Spec}#{tag.Scenario}\");\n"
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
        (consumer / "package.json").write_text('{"private":true,"type":"commonjs"}\n')
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
                "const a=require('@azimuth/annotations');"
                "a.realizes('consumer','starts');console.log(typeof a.covers)",
            ],
            cwd=consumer,
            capture=True,
        )
        require(annotation_result.stdout.strip() == "function", "npm annotation entry point failed")
        source = consumer / "sample.ts"
        source.write_text(
            "import { realizes } from '@azimuth/annotations';\n"
            "export function start(): void { realizes('consumer', 'starts'); }\n"
        )
        manifest = consumer / "manifest.json"
        run(
            [
                "node",
                consumer / "node_modules/@azimuth/emit/dist/cli.js",
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
            any(item["spec"] == "consumer" and item["scenario"] == "starts"
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
                candidate.extractall(consumer)
        result = run([consumer / binary.name, "--version"], capture=True)
        require(version in result.stdout, f"{target}: retained CLI version differs")
    return archive


def image_contract(root, image_id):
    catalog = catalog_at(root)
    return next((item for item in catalog["images"] if item["id"] == image_id), None)


def inspect_image(root, image_id, archive):
    from services.assurance.deployment.qualify import inspect_oci_platforms

    image = image_contract(root, image_id)
    require(image is not None, f"image {image_id!r} is absent")
    observed = inspect_oci_platforms(archive)
    require(observed == sorted(image["platforms"]), f"{image_id}: OCI platform account differs")


def free_port():
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        return listener.getsockname()[1]


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


def smoke_api(tag, platform, suffix):
    network = f"azimuth-release-{suffix}"
    database = f"azimuth-release-db-{suffix}"
    candidate = f"azimuth-release-api-{suffix}"
    port = free_port()
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
                "--network", network, "--publish", f"127.0.0.1:{port}:8080", "--env",
                "DATABASE_URL=postgres://azimuth:rehearsal@" + database + ":5432/azimuth", tag,
            ]
        )
        wait_for_url(f"http://127.0.0.1:{port}/health")
    finally:
        run(["docker", "rm", "--force", candidate, database], check=False)
        run(["docker", "network", "rm", network], check=False)


def smoke_web(tag, platform, suffix):
    candidate = f"azimuth-release-web-{suffix}"
    port = free_port()
    try:
        run(
            [
                "docker", "run", "--detach", "--platform", platform, "--name", candidate,
                "--publish", f"127.0.0.1:{port}:3000", "--env",
                "ASSURANCE_API_URL=http://127.0.0.1:9", tag,
            ]
        )
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

    inspect = commands.add_parser("inspect-image")
    inspect.add_argument("--root", type=Path, default=ROOT)
    inspect.add_argument("--id", required=True)
    inspect.add_argument("--archive", type=Path, required=True)

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
    elif arguments.command == "inspect-image":
        inspect_image(arguments.root, arguments.id, arguments.archive)
        print(f"verified selected platforms in {arguments.archive.name}")
    else:
        smoke_image(arguments.root, arguments.id, arguments.archive)
        print(f"exercised every selected platform in {arguments.archive.name}")


if __name__ == "__main__":
    try:
        main()
    except (CandidateError, QualificationError, subprocess.CalledProcessError) as error:
        raise SystemExit(f"release candidate failed: {error}") from error
