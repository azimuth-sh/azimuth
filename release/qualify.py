#!/usr/bin/env python3

import argparse
import copy
import fnmatch
import hashlib
import json
import subprocess
import tarfile
import tempfile
import tomllib
import zipfile
from pathlib import Path, PurePosixPath
from xml.etree import ElementTree

if __package__:
    from .acceptance import APPROVED_CONTRACT
else:
    from acceptance import APPROVED_CONTRACT


SPEC = "framework/release-artifacts"
CLAIMS = ("first-alpha-contract",)


class QualificationError(Exception):
    pass


def combined_digest(paths):
    account = hashlib.sha256()
    for path in paths:
        account.update(path.read_bytes())
    return account.hexdigest()


def read_json(path):
    return json.loads(path.read_text())


def run(command, root):
    subprocess.run(command, cwd=root, check=True)


def require(condition, message):
    if not condition:
        raise QualificationError(message)


def catalog_at(root):
    return read_json(root / "release/artifacts.json")


def validate_catalog(catalog, root):
    require(catalog.get("format") == "azimuth-release-artifacts", "catalog format is unsupported")
    require(catalog.get("schemaVersion") == 1, "catalog schemaVersion is unsupported")
    release = catalog.get("release", {})
    for field in ("version", "tag", "migrationLine", "license", "repository", "homepage"):
        require(release.get(field), f"release.{field} is required")

    resources = catalog.get("resources", {})
    require(resources.get("format") == "azimuth-cli-resources", "resource format is unsupported")
    require(resources.get("package") in [item.get("id") for item in catalog.get("packages", [])], "resource package is not public")
    resource_manifest = root / resources.get("manifest", "")
    require(resource_manifest.is_file(), "resource manifest does not exist")
    resource_account = read_json(resource_manifest)
    require(resource_account.get("format") == resources["format"], "resource manifest format differs from catalog")
    require(resource_account.get("releaseVersion") == release["version"], "resource release differs from catalog")
    require(resource_account.get("migrationLine") == release["migrationLine"], "resource migration line differs from catalog")
    for collection in ("skills", "references", "templates", "migrations"):
        require(resource_account.get(collection), f"resource manifest {collection} is empty")
    resource_root = resource_manifest.parent
    for skill in resource_account["skills"]:
        require((resource_root / "skills" / skill / "SKILL.md").is_file(), f"resource skill is missing: {skill}")
        require((resource_root / "skills" / skill / "agents/openai.yaml").is_file(), f"resource skill metadata is missing: {skill}")
    for reference in resource_account["references"]:
        require((resource_root / "references" / f"{reference}.md").is_file(), f"resource reference is missing: {reference}")
    for template in resource_account["templates"]:
        require((resource_root / "templates" / template).is_file(), f"resource template is missing: {template}")
    for migration in resource_account["migrations"]:
        require((resource_root / "migrations" / f"{migration}.md").is_file(), f"resource migration is missing: {migration}")

    protocols = catalog.get("protocols", [])
    require(protocols, "catalog declares no protocol compatibility")
    protocol_ids = [item.get("id") for item in protocols]
    require(len(protocol_ids) == len(set(protocol_ids)), "protocol ids are not unique")
    for protocol in protocols:
        require(isinstance(protocol.get("version"), int) and protocol["version"] > 0, f"{protocol.get('id')}: invalid protocol version")
        require(isinstance(protocol.get("produces"), bool), f"{protocol.get('id')}: produces must be boolean")
        require(isinstance(protocol.get("accepts"), bool), f"{protocol.get('id')}: accepts must be boolean")
        require(protocol["produces"] or protocol["accepts"], f"{protocol.get('id')}: protocol has no direction")

    packages = catalog.get("packages", [])
    images = catalog.get("images", [])
    require(packages, "catalog declares no public packages")
    require(images, "catalog declares no public images")
    package_ids = [item.get("id") for item in packages]
    image_ids = [item.get("id") for item in images]
    identities = [item.get("identity") for item in packages + images]
    require(len(package_ids) == len(set(package_ids)), "package ids are not unique")
    require(len(image_ids) == len(set(image_ids)), "image ids are not unique")
    require(len(identities) == len(set(identities)), "public identities are not unique")

    experimental = [PurePosixPath(path) for path in catalog.get("experimentalSource", [])]
    require(experimental, "catalog declares no experimental source roots")
    for path in experimental:
        require((root / path).is_dir(), f"experimental source root does not exist: {path}")

    for package in packages:
        ecosystem = package.get("ecosystem")
        require(ecosystem in {"cargo", "nuget", "npm"}, f"{package.get('id')}: unknown ecosystem")
        manifest = PurePosixPath(package.get("manifest", ""))
        for source_root in experimental:
            require(
                not manifest.is_relative_to(source_root),
                f"{package.get('id')}: public package is inside experimental source {source_root}",
            )
        require((root / manifest).is_file(), f"{package.get('id')}: manifest does not exist")
        require(package.get("requiredFiles"), f"{package.get('id')}: requiredFiles is empty")
        require(package.get("allowedFiles"), f"{package.get('id')}: allowedFiles is empty")

    for image in images:
        context = Path(image.get("context", ""))
        require((root / context).is_dir(), f"{image.get('id')}: build context does not exist")
        dockerfile = Path(image.get("dockerfile", ""))
        require((root / dockerfile).is_file(), f"{image.get('id')}: Dockerfile does not exist")
        version_manifest = Path(image.get("versionManifest", ""))
        require(
            (root / version_manifest).is_file(),
            f"{image.get('id')}: version manifest does not exist",
        )
        require(image.get("platforms"), f"{image.get('id')}: platforms is empty")

    binary = catalog.get("nativeBinaries", {})
    require(binary.get("identity"), "nativeBinaries.identity is required")
    require(binary.get("targets"), "nativeBinaries.targets is empty")
    require(catalog.get("supportedSurfaces"), "catalog declares no supported surfaces")


def native_package_metadata(package, root):
    manifest = root / package["manifest"]
    ecosystem = package["ecosystem"]
    if ecosystem == "cargo":
        data = tomllib.loads(manifest.read_text())["package"]
        return {
            "identity": data["name"],
            "version": data["version"],
            "license": data["license"],
            "repository": data["repository"],
            "homepage": data["homepage"],
        }
    if ecosystem == "npm":
        data = read_json(manifest)
        repository = data.get("repository", {})
        repository_url = repository.get("url") if isinstance(repository, dict) else repository
        require(data.get("private") is not True, f"{package['id']}: npm package is private")
        return {
            "identity": data["name"],
            "version": data["version"],
            "license": data["license"],
            "repository": repository_url.replace("git+", "").removesuffix(".git"),
            "homepage": data["homepage"],
        }

    project = ElementTree.parse(manifest).getroot()
    values = {node.tag: (node.text or "").strip() for node in project.iter()}
    return {
        "identity": values.get("PackageId"),
        "version": values.get("Version"),
        "license": values.get("PackageLicenseExpression"),
        "repository": values.get("RepositoryUrl"),
        "homepage": values.get("PackageProjectUrl"),
    }


def validate_source_metadata(catalog, root):
    release = catalog["release"]
    for package in catalog["packages"]:
        metadata = native_package_metadata(package, root)
        expected = {
            "identity": package["identity"],
            "version": release["version"],
            "license": release["license"],
            "repository": release["repository"],
            "homepage": release["homepage"],
        }
        for field, value in expected.items():
            require(
                metadata.get(field) == value,
                f"{package['id']}: {field} is {metadata.get(field)!r}, expected {value!r}",
            )

    for image in catalog["images"]:
        source = (root / image["dockerfile"]).read_text()
        require(
            f"ARG AZIMUTH_VERSION={release['version']}" in source,
            f"{image['id']}: Dockerfile version differs from the catalog",
        )
        require(
            f'org.opencontainers.image.source="{release["repository"]}"' in source,
            f"{image['id']}: Dockerfile source label differs from the catalog",
        )
        require(
            f'org.opencontainers.image.url="{release["homepage"]}"' in source,
            f"{image['id']}: Dockerfile homepage label differs from the catalog",
        )
        require(
            f'org.opencontainers.image.licenses="{release["license"]}"' in source,
            f"{image['id']}: Dockerfile license label differs from the catalog",
        )
        manifest = root / image["versionManifest"]
        if manifest.suffix == ".toml":
            version = tomllib.loads(manifest.read_text())["workspace"]["package"]["version"]
        else:
            version = read_json(manifest)["version"]
        require(
            version == release["version"],
            f"{image['id']}: version manifest differs from the catalog",
        )

    readme = (root / "README.md").read_text()
    for phrase in (
        "Linux x64",
        "macOS ARM64",
        "Windows x64",
        "Linux AMD64",
        "Linux ARM64",
        "experimental source",
    ):
        require(phrase in readme, f"README support account omits {phrase!r}")


def approved_contract_differences(catalog, approved):
    release = catalog.get("release", {})
    actual = {
        "version": release.get("version"),
        "tag": release.get("tag"),
        "migrationLine": release.get("migrationLine"),
        "license": release.get("license"),
        "repository": release.get("repository"),
        "homepage": release.get("homepage"),
        "identities": sorted(
            item.get("identity") for item in catalog.get("packages", []) + catalog.get("images", [])
        ),
        "nativeTargets": sorted(catalog.get("nativeBinaries", {}).get("targets", [])),
        "imagePlatforms": {
            image.get("identity"): sorted(image.get("platforms", []))
            for image in catalog.get("images", [])
        },
        "supportedSurfaces": sorted(catalog.get("supportedSurfaces", [])),
        "experimentalSource": sorted(catalog.get("experimentalSource", [])),
    }
    differences = []
    for field, value in approved.items():
        expected = sorted(value) if isinstance(value, list) else value
        if actual.get(field) != expected:
            differences.append(field)
    return differences


def validate_approved_contract(catalog):
    differences = approved_contract_differences(catalog, APPROVED_CONTRACT)
    require(not differences, f"catalog differs from approved contract: {differences}")


def cargo_candidate(package, root, allow_dirty):
    command = ["cargo", "package", "--manifest-path", package["manifest"]]
    if allow_dirty:
        command.append("--allow-dirty")
    run(command, root)
    release = catalog_at(root)["release"]
    archive = f"{package['identity']}-{release['version']}.crate"
    return root / "tools/azimuth/target/package" / archive


def nuget_candidate(package, root, destination):
    run(
        [
            "dotnet",
            "pack",
            package["manifest"],
            "--configuration",
            "Release",
            "--output",
            str(destination),
            "--nologo",
        ],
        root,
    )
    version = catalog_at(root)["release"]["version"]
    return destination / f"{package['identity']}.{version}.nupkg"


def npm_candidate(package, root, destination):
    package_root = (root / package["manifest"]).parent
    run(["npm", "run", "build", "--silent"], package_root)
    run(["npm", "pack", "--silent", "--pack-destination", str(destination)], package_root)
    candidates = sorted(destination.glob("*.tgz"), key=lambda path: path.stat().st_mtime_ns)
    require(candidates, f"{package['id']}: npm pack produced no archive")
    return candidates[-1]


def archive_files(candidate, ecosystem):
    if ecosystem == "nuget":
        with zipfile.ZipFile(candidate) as archive:
            return sorted(name for name in archive.namelist() if not name.endswith("/"))
    with tarfile.open(candidate, "r:gz") as archive:
        return sorted(member.name for member in archive.getmembers() if member.isfile())


def path_matches(path, pattern):
    return path == pattern or fnmatch.fnmatchcase(path, pattern)


def validate_file_set(package, files):
    for required in package["requiredFiles"]:
        require(
            any(path_matches(path, required) for path in files),
            f"{package['id']}: package omits required file {required}",
        )
    unexpected = [
        path
        for path in files
        if not any(path_matches(path, allowed) for allowed in package["allowedFiles"])
    ]
    require(not unexpected, f"{package['id']}: package contains undeclared files {unexpected}")
    forbidden_parts = {
        ".git",
        ".env",
        "bin",
        "fixture",
        "fixtures",
        "node_modules",
        "obj",
        "test",
        "tests",
    }
    forbidden = [
        path
        for path in files
        if any(part.lower() in forbidden_parts for part in PurePosixPath(path).parts)
    ]
    require(not forbidden, f"{package['id']}: package contains forbidden files {forbidden}")


def stable_content_account(package, files):
    counts = {}
    for path in files:
        pattern = next(
            allowed for allowed in package["allowedFiles"] if path_matches(path, allowed)
        )
        counts[pattern] = counts.get(pattern, 0) + 1
    return [
        {"allowed": pattern, "count": counts[pattern]}
        for pattern in package["allowedFiles"]
        if pattern in counts
    ]


def archive_metadata(candidate, ecosystem):
    if ecosystem == "nuget":
        with zipfile.ZipFile(candidate) as archive:
            nuspec = next(name for name in archive.namelist() if name.endswith(".nuspec"))
            root = ElementTree.fromstring(archive.read(nuspec))
            values = {
                node.tag.rsplit("}", 1)[-1]: (node.text or "").strip()
                for node in root.iter()
            }
            repository = next(
                (node for node in root.iter() if node.tag.rsplit("}", 1)[-1] == "repository"),
                None,
            )
            return {
                "identity": values.get("id"),
                "version": values.get("version"),
                "license": values.get("license"),
                "repository": repository.get("url") if repository is not None else None,
            }
    with tarfile.open(candidate, "r:gz") as archive:
        manifest_name = next(
            member.name
            for member in archive.getmembers()
            if member.isfile() and member.name.endswith("/Cargo.toml")
        ) if ecosystem == "cargo" else "package/package.json"
        source = archive.extractfile(manifest_name).read()
    if ecosystem == "cargo":
        data = tomllib.loads(source.decode())["package"]
        return {
            "identity": data["name"],
            "version": data["version"],
            "license": data["license"],
            "repository": data["repository"],
        }
    data = json.loads(source)
    repository = data.get("repository", {})
    repository_url = repository.get("url") if isinstance(repository, dict) else repository
    return {
        "identity": data["name"],
        "version": data["version"],
        "license": data["license"],
        "repository": repository_url.replace("git+", "").removesuffix(".git"),
    }


def validate_archive_metadata(package, metadata, release):
    expected = {
        "identity": package["identity"],
        "version": release["version"],
        "license": release["license"],
        "repository": release["repository"],
    }
    for field, value in expected.items():
        require(
            metadata.get(field) == value,
            f"{package['id']}: packed {field} is {metadata.get(field)!r}, expected {value!r}",
        )


def qualify_packages(catalog, root, allow_dirty):
    results = []
    with tempfile.TemporaryDirectory(prefix="azimuth-release-") as temporary:
        destination = Path(temporary)
        for package in catalog["packages"]:
            ecosystem = package["ecosystem"]
            if ecosystem == "cargo":
                candidate = cargo_candidate(package, root, allow_dirty)
            elif ecosystem == "nuget":
                candidate = nuget_candidate(package, root, destination)
            else:
                candidate = npm_candidate(package, root, destination)
            require(candidate.is_file(), f"{package['id']}: package archive does not exist")
            files = archive_files(candidate, ecosystem)
            validate_file_set(package, files)
            metadata = archive_metadata(candidate, ecosystem)
            validate_archive_metadata(package, metadata, catalog["release"])
            results.append(
                {
                    "id": package["id"],
                    "identity": package["identity"],
                    "ecosystem": ecosystem,
                    "archive": candidate.name,
                    "metadata": metadata,
                    "contents": stable_content_account(package, files),
                }
            )
    return results


def write_linkage(root, output_root):
    qualifier = root / "release/qualify.py"
    acceptance = root / "release/acceptance.py"
    qualifier_fingerprint = f"sha256:{combined_digest([qualifier, acceptance])}"
    linkage = {
        "realizes": [
            {
                "spec": SPEC,
                "claim": claim,
                "site": site,
                "file": "release/qualify.py",
                "lang": "python",
                "source_fingerprint": qualifier_fingerprint,
            }
            for claim in CLAIMS
            for site in ["qualify"]
        ],
        "check_implementations": [],
        "mechanism_implementations": [],
        "class_members": [],
        "enumerations": [],
        "artifacts": [
            {
                "id": "release-artifact-contract",
                "kind": "release-catalog",
                "file": "release/artifacts.json",
            }
        ],
    }
    (output_root / "linkage.json").write_text(json.dumps(linkage, indent=2) + "\n")


def qualify(root, output_root, allow_dirty):
    catalog = catalog_at(root)
    validate_catalog(catalog, root)
    validate_approved_contract(catalog)
    validate_source_metadata(catalog, root)
    packages = qualify_packages(catalog, root, allow_dirty)
    output_root.mkdir(parents=True, exist_ok=True)
    qualification = {
        "format": "azimuth-release-qualification",
        "schemaVersion": 1,
        "release": copy.deepcopy(catalog["release"]),
        "packages": packages,
        "images": copy.deepcopy(catalog["images"]),
        "nativeBinaries": copy.deepcopy(catalog["nativeBinaries"]),
        "resources": copy.deepcopy(catalog["resources"]),
        "protocols": copy.deepcopy(catalog["protocols"]),
        "supportedSurfaces": copy.deepcopy(catalog["supportedSurfaces"]),
        "experimentalSource": copy.deepcopy(catalog["experimentalSource"]),
    }
    (output_root / "qualification.json").write_text(json.dumps(qualification, indent=2) + "\n")
    write_linkage(root, output_root)
    return qualification


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parent.parent)
    parser.add_argument("--out", type=Path)
    parser.add_argument("--allow-dirty", action="store_true")
    arguments = parser.parse_args()
    root = arguments.root.resolve()
    output = arguments.out or root / ".azimuth/release"
    qualification = qualify(root, output, arguments.allow_dirty)
    print(
        f"qualified {len(qualification['packages'])} package candidate(s), "
        f"{len(qualification['images'])} image contract(s), and "
        f"{len(qualification['nativeBinaries']['targets'])} native target(s)"
    )


if __name__ == "__main__":
    try:
        main()
    except QualificationError as error:
        raise SystemExit(f"release qualification failed: {error}")
