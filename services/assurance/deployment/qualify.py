#!/usr/bin/env python3

import argparse
import hashlib
import json
import os
import secrets
import socket
import subprocess
import tarfile
import tempfile
import time
import urllib.request
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
ASSURANCE_ROOT = ROOT / "services/assurance"
COMPOSE_FILE = ASSURANCE_ROOT / "docker-compose.yml"
README_FILE = ASSURANCE_ROOT / "README.md"
CATALOG_FILE = ROOT / "release/artifacts.json"
MIGRATION_ROOT = ASSURANCE_ROOT / "server/migrations"
OUTPUT_FILE = ROOT / ".azimuth/release/private-deployment.json"
QUALIFICATION_SECRET = "private-qualification-9f4c8b6e"
REQUIRED_DOCUMENTATION = (
    "operator-controlled trusted reverse proxy, SSH tunnel or VPN",
    "does not add application authentication",
    "not ready for direct internet exposure",
    "Downgrades are unsupported",
    "later schema-changing release",
)


class QualificationError(Exception):
    pass


def require(condition, message):
    if not condition:
        raise QualificationError(message)


def compose_command(arguments, environment, capture=False):
    command = ["docker", "compose", "-f", str(COMPOSE_FILE), *arguments]
    return subprocess.run(
        command,
        cwd=ASSURANCE_ROOT,
        env=environment,
        check=True,
        capture_output=capture,
        text=True,
    )


def resolved_configuration():
    missing_environment = os.environ.copy()
    missing_environment.pop("ASSURANCE_POSTGRES_PASSWORD", None)
    missing = subprocess.run(
        ["docker", "compose", "-f", str(COMPOSE_FILE), "config", "--format", "json"],
        cwd=ASSURANCE_ROOT,
        env=missing_environment,
        capture_output=True,
        text=True,
    )
    require(missing.returncode != 0, "Compose resolves without a deployment-owned password")

    environment = missing_environment | {
        "ASSURANCE_POSTGRES_PASSWORD": QUALIFICATION_SECRET,
        "ASSURANCE_API_PORT": "18080",
        "ASSURANCE_WEB_PORT": "13000",
    }
    result = compose_command(["config", "--format", "json"], environment, capture=True)
    return json.loads(result.stdout)


def validate_documentation(documentation):
    normalized = " ".join(documentation.split())
    for statement in REQUIRED_DOCUMENTATION:
        require(statement in normalized, f"deployment account omits {statement!r}")


def validate_configuration(configuration, documentation):
    services = configuration.get("services", {})
    require(set(services) == {"postgres", "api", "web"}, "unexpected Compose service account")

    postgres = services["postgres"]
    require(not postgres.get("ports"), "PostgreSQL publishes a host port")
    postgres_password = postgres.get("environment", {}).get("POSTGRES_PASSWORD")
    require(
        postgres_password == QUALIFICATION_SECRET,
        "PostgreSQL does not use the deployment-owned password",
    )
    volumes = postgres.get("volumes", [])
    require(
        any(
            volume.get("type") == "volume"
            and volume.get("source") == "assurance-data"
            and volume.get("target") == "/var/lib/postgresql/data"
            for volume in volumes
        ),
        "PostgreSQL does not use the declared persistent volume",
    )

    database_url = services["api"].get("environment", {}).get("DATABASE_URL", "")
    require(QUALIFICATION_SECRET in database_url, "API database credential differs from Postgres")
    require("@postgres:5432/azimuth" in database_url, "API bypasses the private database service")

    for service_name in ("api", "web"):
        ports = services[service_name].get("ports", [])
        require(ports, f"{service_name} has no operator entry point")
        for port in ports:
            require(
                port.get("host_ip") == "127.0.0.1",
                f"{service_name} publishes outside host loopback",
            )
        require(
            "default" in services[service_name].get("networks", {}),
            f"{service_name} is absent from the private Compose network",
        )

    validate_documentation(documentation)


def migration_account():
    migrations = sorted(path.name for path in MIGRATION_ROOT.glob("*.sql"))
    require(migrations, "assurance server has no migrations")
    require(
        not any(name.endswith(".down.sql") for name in migrations),
        "downgrade migration is present in the forward-only migration set",
    )
    prefixes = [name.split("_", 1)[0] for name in migrations]
    require(prefixes == sorted(prefixes), "migration filenames are not in forward order")
    main_source = (ASSURANCE_ROOT / "server/src/main.rs").read_text()
    require("migrate(&pool).await" in main_source, "API startup does not apply migrations")
    return migrations


def migration_versions(migrations):
    return [int(name.split("_", 1)[0]) for name in migrations]


def applied_migration_versions(environment):
    result = compose_command(
        [
            "exec",
            "--no-TTY",
            "postgres",
            "psql",
            "--username",
            "azimuth",
            "--dbname",
            "azimuth",
            "--tuples-only",
            "--no-align",
            "--command",
            "SELECT version FROM _sqlx_migrations WHERE success ORDER BY version",
        ],
        environment,
        capture=True,
    )
    return [int(line) for line in result.stdout.splitlines() if line.strip()]


def free_loopback_port():
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        return listener.getsockname()[1]


def get_json(url, attempts=60):
    last_error = None
    for _ in range(attempts):
        try:
            with urllib.request.urlopen(url, timeout=2) as response:
                return json.loads(response.read())
        except Exception as error:  # The last concrete connection failure is reported below.
            last_error = error
            time.sleep(1)
    raise QualificationError(f"service did not become readable at {url}: {last_error}")


def wait_for_web(url, attempts=60):
    last_error = None
    for _ in range(attempts):
        try:
            with urllib.request.urlopen(url, timeout=2) as response:
                require(response.status == 200, "web entry point returned a non-success status")
                return
        except Exception as error:  # The last concrete connection failure is reported below.
            last_error = error
            time.sleep(1)
    raise QualificationError(f"web entry point did not become readable at {url}: {last_error}")


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"))


def demonstrate_lifecycle():
    api_port = free_loopback_port()
    web_port = free_loopback_port()
    project = f"azimuth-private-qualification-{os.getpid()}"
    environment = os.environ.copy() | {
        "ASSURANCE_POSTGRES_PASSWORD": secrets.token_hex(24),
        "ASSURANCE_API_PORT": str(api_port),
        "ASSURANCE_WEB_PORT": str(web_port),
        "COMPOSE_PROJECT_NAME": project,
    }
    api_url = f"http://127.0.0.1:{api_port}"
    web_url = f"http://127.0.0.1:{web_port}"
    transitions = []
    expected_migrations = migration_versions(migration_account())
    try:
        compose_command(["up", "--detach", "--build"], environment)
        get_json(f"{api_url}/health")
        wait_for_web(web_url)
        require(
            applied_migration_versions(environment) == expected_migrations,
            "PostgreSQL migration ledger differs from the repository migration set",
        )
        seed_environment = environment | {"ASSURANCE_URL": api_url}
        subprocess.run(
            ["./seed-demo.sh"],
            cwd=ASSURANCE_ROOT,
            env=seed_environment,
            check=True,
        )
        expected = canonical(get_json(f"{api_url}/v1/projects/checkout/snapshot"))

        compose_command(["stop"], environment)
        compose_command(["start"], environment)
        get_json(f"{api_url}/health")
        wait_for_web(web_url)
        require(
            canonical(get_json(f"{api_url}/v1/projects/checkout/snapshot")) == expected,
            "ledger history changed across stop and start",
        )
        transitions.append("stop-start")

        compose_command(
            ["up", "--detach", "--force-recreate", "postgres", "api", "web"],
            environment,
        )
        get_json(f"{api_url}/health")
        wait_for_web(web_url)
        require(
            canonical(get_json(f"{api_url}/v1/projects/checkout/snapshot")) == expected,
            "ledger history changed across service-container recreation",
        )
        require(
            applied_migration_versions(environment) == expected_migrations,
            "service recreation changed the successful migration account",
        )
        transitions.append("service-recreation")
        return {
            "outcome": "passed",
            "transitions": transitions,
            "migrationVersions": expected_migrations,
            "snapshotSha256": hashlib.sha256(expected.encode()).hexdigest(),
        }
    finally:
        compose_command(["down", "--volumes", "--remove-orphans"], environment)


def platforms_from_oci_index(index, load_descriptor):
    platforms = set()
    for descriptor in index.get("manifests", []):
        if descriptor.get("mediaType") == "application/vnd.oci.image.index.v1+json":
            platforms.update(platforms_from_oci_index(load_descriptor(descriptor), load_descriptor))
            continue
        platform = descriptor.get("platform", {})
        operating_system = platform.get("os")
        architecture = platform.get("architecture")
        if operating_system and architecture and "unknown" not in (operating_system, architecture):
            platforms.add(f"{operating_system}/{architecture}")
    return sorted(platforms)


def inspect_oci_platforms(archive):
    with tarfile.open(archive) as candidate:
        index_file = candidate.extractfile("index.json")
        require(index_file is not None, "OCI candidate has no index.json")
        index = json.load(index_file)

        def load_descriptor(descriptor):
            digest = descriptor["digest"].removeprefix("sha256:")
            content = candidate.extractfile(f"blobs/sha256/{digest}")
            require(content is not None, f"OCI candidate omits descriptor {descriptor['digest']}")
            return json.load(content)

        return platforms_from_oci_index(index, load_descriptor)


def build_selected_images():
    catalog = json.loads(CATALOG_FILE.read_text())
    results = []
    subprocess.run(["docker", "buildx", "inspect", "--bootstrap"], check=True)
    with tempfile.TemporaryDirectory(prefix="azimuth-private-images-") as destination:
        destination_root = Path(destination)
        for image in catalog.get("images", []):
            platforms = sorted(image.get("platforms", []))
            require(platforms, f"{image.get('id')}: no selected platforms")
            archive = destination_root / f"{image['id']}.oci.tar"
            subprocess.run(
                [
                    "docker",
                    "buildx",
                    "build",
                    "--platform",
                    ",".join(platforms),
                    "--file",
                    str(ROOT / image["dockerfile"]),
                    "--output",
                    f"type=oci,dest={archive}",
                    str(ROOT / image["context"]),
                ],
                cwd=ROOT,
                check=True,
            )
            observed = inspect_oci_platforms(archive)
            require(observed == platforms, f"{image['id']}: OCI platform account differs")
            results.append(
                {
                    "id": image["id"],
                    "identity": image["identity"],
                    "platforms": observed,
                    "candidateSha256": hashlib.sha256(archive.read_bytes()).hexdigest(),
                }
            )
    return results


def qualify(run_lifecycle, run_images):
    configuration = resolved_configuration()
    validate_configuration(configuration, README_FILE.read_text())
    result = {
        "format": "azimuth-private-deployment-qualification",
        "schemaVersion": 1,
        "configuration": {
            "credentialSource": "ASSURANCE_POSTGRES_PASSWORD",
            "databaseHostPorts": [],
            "applicationHostAddresses": ["127.0.0.1"],
            "persistentVolume": "assurance-data",
        },
        "migrations": migration_account(),
        "lifecycle": demonstrate_lifecycle() if run_lifecycle else {"outcome": "not-run"},
        "images": build_selected_images() if run_images else [],
    }
    OUTPUT_FILE.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT_FILE.write_text(json.dumps(result, indent=2) + "\n")
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--lifecycle", action="store_true")
    parser.add_argument("--images", action="store_true")
    arguments = parser.parse_args()
    result = qualify(arguments.lifecycle, arguments.images)
    lifecycle = result["lifecycle"]["outcome"]
    print(
        "private deployment qualified: "
        f"configuration=passed lifecycle={lifecycle} images={len(result['images'])}"
    )


if __name__ == "__main__":
    try:
        main()
    except (QualificationError, subprocess.CalledProcessError) as error:
        raise SystemExit(f"private deployment qualification failed: {error}") from error
