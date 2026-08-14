import copy
import unittest

from services.assurance.deployment.qualify import (
    QUALIFICATION_SECRET,
    QualificationError,
    migration_account,
    migration_versions,
    platforms_from_oci_index,
    validate_configuration,
)


DOCUMENTATION = """
operator-controlled trusted reverse proxy, SSH tunnel or VPN
does not add application authentication
not ready for direct internet exposure
Downgrades are unsupported
later schema-changing release
"""


def valid_configuration():
    return {
        "services": {
            "postgres": {
                "environment": {"POSTGRES_PASSWORD": QUALIFICATION_SECRET},
                "networks": {"default": None},
                "volumes": [
                    {
                        "type": "volume",
                        "source": "assurance-data",
                        "target": "/var/lib/postgresql/data",
                    }
                ],
            },
            "api": {
                "environment": {
                    "DATABASE_URL": (
                        f"postgres://azimuth:{QUALIFICATION_SECRET}@postgres:5432/azimuth"
                    )
                },
                "networks": {"default": None},
                "ports": [{"host_ip": "127.0.0.1", "target": 8080}],
            },
            "web": {
                "networks": {"default": None},
                "ports": [{"host_ip": "127.0.0.1", "target": 3000}],
            },
        }
    }


class PrivateDeploymentQualificationTests(unittest.TestCase):
    def test_complete_private_configuration_passes(self):
        validate_configuration(valid_configuration(), DOCUMENTATION)

    def test_postgres_host_port_fails(self):
        configuration = valid_configuration()
        configuration["services"]["postgres"]["ports"] = [{"published": "5432"}]
        with self.assertRaisesRegex(QualificationError, "PostgreSQL publishes"):
            validate_configuration(configuration, DOCUMENTATION)

    def test_non_loopback_application_port_fails(self):
        configuration = valid_configuration()
        configuration["services"]["api"]["ports"][0]["host_ip"] = "0.0.0.0"
        with self.assertRaisesRegex(QualificationError, "outside host loopback"):
            validate_configuration(configuration, DOCUMENTATION)

    def test_repository_password_fails(self):
        configuration = valid_configuration()
        configuration["services"]["postgres"]["environment"]["POSTGRES_PASSWORD"] = "azimuth"
        with self.assertRaisesRegex(QualificationError, "deployment-owned password"):
            validate_configuration(configuration, DOCUMENTATION)

    def test_api_password_mismatch_fails(self):
        configuration = valid_configuration()
        configuration["services"]["api"]["environment"]["DATABASE_URL"] = (
            "postgres://azimuth:different@postgres:5432/azimuth"
        )
        with self.assertRaisesRegex(QualificationError, "differs from Postgres"):
            validate_configuration(configuration, DOCUMENTATION)

    def test_missing_persistent_volume_fails(self):
        configuration = valid_configuration()
        configuration["services"]["postgres"]["volumes"] = []
        with self.assertRaisesRegex(QualificationError, "persistent volume"):
            validate_configuration(configuration, DOCUMENTATION)

    def test_each_boundary_statement_is_required(self):
        for line in DOCUMENTATION.strip().splitlines():
            with self.subTest(line=line):
                mutated = DOCUMENTATION.replace(line, "")
                with self.assertRaisesRegex(QualificationError, "deployment account omits"):
                    validate_configuration(copy.deepcopy(valid_configuration()), mutated)

    def test_migration_account_is_forward_only(self):
        self.assertEqual(migration_account(), ["0001_assurance_ledger.sql"])
        self.assertEqual(migration_versions(migration_account()), [1])

    def test_nested_oci_index_reports_images_and_ignores_attestations(self):
        root = {
            "manifests": [
                {
                    "mediaType": "application/vnd.oci.image.index.v1+json",
                    "digest": "sha256:nested",
                }
            ]
        }
        nested = {
            "manifests": [
                {"platform": {"os": "linux", "architecture": "amd64"}},
                {"platform": {"os": "linux", "architecture": "arm64"}},
                {"platform": {"os": "unknown", "architecture": "unknown"}},
            ]
        }
        self.assertEqual(
            platforms_from_oci_index(root, lambda descriptor: nested),
            ["linux/amd64", "linux/arm64"],
        )


if __name__ == "__main__":
    unittest.main()
