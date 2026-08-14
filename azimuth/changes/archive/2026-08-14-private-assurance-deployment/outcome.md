# Outcome: private-assurance-deployment

Status: accepted

## Result

Azimuth now has one private single-team assurance deployment profile. The deployment supplies the
PostgreSQL credential, PostgreSQL has no host port, and the API and web ports bind only to
loopback. The operator runbook assigns ingress trust to a reverse proxy, SSH tunnel or VPN and
states separately that the profile provides neither application authentication nor public
internet hardening.

The composed lifecycle preserves the externally readable assurance account through stop/start and
forced recreation of PostgreSQL, API and web while retaining only the named database volume. API
startup applies the complete tracked SQLx migration set to an empty database and observes the same
successful migration account after recreation.

Both assurance images build as OCI candidates for every platform selected by the release catalog:
Linux AMD64 and ARM64. Qualification performs no registry output or publication.

## Acceptance evidence

- Nine private-deployment qualifier cases passed, including independent violations of the secret,
  host-binding, persistence, documentation, migration and OCI-index contracts.
- The real three-service Compose lifecycle created an empty named volume, registered accepted
  assurance history through HTTP, compared its canonical public snapshot after stop/start, and
  compared it again after recreating all three service containers.
- The lifecycle observed SQLx migration version 1 before and after recreation, equal to the one
  tracked migration derived from the repository.
- BuildKit completed both catalog-selected assurance images for Linux AMD64 and ARM64 and exported
  two local OCI candidates without pushing them.
- Release qualification still accepted 5 package candidates, 2 image contracts and 3 native
  targets; experimental-source qualification still accepted 11 roots, 49 executable inputs and 3
  immutable citations.
- The complete repository gate passed. The canonical model contained 16 claims in 3 specs with no
  holes, errors or warnings.

## Departures

The original lifecycle description named recreation of the API and service containers. Agent-tier
inspection found that such a check could pass without exercising database-volume persistence. The
accepted evidence therefore recreates PostgreSQL, API and web together and retains only the named
volume. This strengthens the claimed lifecycle rather than changing its intended boundary.

The initial migration check inspected tracked filenames and the startup call. That could not show
that PostgreSQL had applied every migration. The accepted oracle queries SQLx's migration ledger
and compares its successful ordered versions with the complete tracked population before and after
recreation.

The first OCI reader assumed BuildKit exported one flat image index. The actual multi-platform
archive contained nested indexes and provenance attestations with `unknown/unknown` platforms. The
accepted reader traverses nested indexes, excludes attestation manifests from the runnable-platform
population and has a regression case for that exporter shape.

## Residual decisions

- The repository can constrain host bindings but cannot establish that an operator's reverse
  proxy, tunnel or VPN is trusted. Revisit this boundary before any non-loopback or public exposure.
- A retained named volume is not backup or disaster recovery. Add backup-and-restore evidence
  before the ledger becomes a non-reconstructable system of record.
- No earlier schema exists, so this change cannot demonstrate cross-version preservation. Every
  later schema-changing release must demonstrate old-image to new-image migration before claiming
  forward compatibility.
- The profile has no application authentication, user authorization or tenant isolation and is
  not suitable for direct internet exposure.

## Measurements

- resolved services: 3;
- PostgreSQL host ports: 0;
- loopback-only application host bindings: 2;
- demonstrated lifecycle transitions: 2;
- service containers recreated while retaining the volume: 3;
- successful tracked migration versions before and after recreation: 1 of 1;
- selected image/platform build candidates: 4 across 2 images;
- private-deployment qualifier cases: 9; and
- accepted-model account: 16 claims in 3 specs, with 0 holes, errors or warnings.
