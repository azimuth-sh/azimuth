# Design: framework/assurance-deployment

## Claim: private-network-containment
Mechanism: private-compose-profile
Enforcement: guard
Binding: private-deployment-qualification

Compose requires the deployment credential during interpolation, keeps PostgreSQL on its internal
network and binds the two application entry points to host loopback. A qualification guard reads
the resolved configuration and the operator account together. Removing the guard would let port,
secret and documentation boundaries drift independently.

## Claim: assurance-ledger-durability
Mechanism: retained-ledger-lifecycle
Enforcement: choke-point
Binding: private-deployment-lifecycle

PostgreSQL owns a named volume while API startup applies embedded SQLx migrations before serving.
The lifecycle test writes through the public API, snapshots externally readable history, crosses
stop, start and forced service recreation, and compares the public representation afterward.
Removing the test would leave volume declaration and application behavior as unrelated assertions.

## Claim: selected-image-platforms-build
Mechanism: catalog-derived-image-matrix
Enforcement: guard
Binding: private-deployment-image-build

The qualification command derives Dockerfiles and platforms from the release catalog and asks
BuildKit to produce OCI candidates without pushing them. Removing the guard would let the catalog
claim architectures that the Dockerfiles cannot build.

## Residue

Loopback binding is network containment on one host, not authentication or internet hardening. It
does not protect an operator who exposes the ports through an untrusted proxy or tunnel. A named
volume does not provide backup, restore, high availability or disaster recovery. This first schema
can demonstrate current-image recreation and idempotent startup migration, but only a later
schema-changing release can demonstrate old-image to new-image preservation.
