# Verification: framework/assurance-deployment

## Claim: deployment-owns-secrets
Scope: component
Quantification: universal
Oracle: direct

The guard resolves the real Compose profile with and without a supplied credential and inspects
every database credential input. Resolution without the input must fail.

## Claim: host-bindings-match-private-boundary
Scope: component
Quantification: universal
Oracle: direct

The guard inspects every resolved service port. PostgreSQL must expose none to the host and every
application host address must be loopback.

## Claim: containment-is-not-authentication
Quantification: universal
Oracle: direct

The documentation check requires separate statements for the trusted network boundary, absent
application authentication and absent direct-internet readiness.

## Claim: history-survives-current-service-recreation
Scope: e2e
Quantification: universal
Oracle: direct

The demonstration uses real application processes and PostgreSQL. It ranges over every documented
service lifecycle transition: stop, start and forced recreation. It compares externally
readable accepted account and execution history before and after each transition.

## Claim: retained-database-migrates-forward
Scope: component
Quantification: universal
Oracle: direct

The lifecycle starts with an empty persistent volume and depends on API startup to apply every
tracked migration. It restarts against the retained volume and rejects a documented downgrade
path. Cross-version preservation remains residual until a second schema exists.

## Claim: complete-selected-image-matrix-builds
Scope: component
Quantification: universal
Oracle: direct

The candidate population is derived from every assurance image and platform in the release
catalog. BuildKit must complete each OCI build without a registry output or push.

## Residual: private-boundary-operator-dependency
Accepted: private single-host dogfood; revisit before any non-loopback or public exposure

The profile cannot establish whether an operator's proxy, tunnel or VPN is trusted. Qualification
detects the repository-owned host boundary only; operational review owns the external boundary.

## Residual: durable-storage-is-not-recovery
Accepted: first private dogfood; revisit before the ledger becomes a non-reconstructable record

The lifecycle detects service recreation loss, not host-volume loss, corruption or operator
error. Backup and restore remain outside the first private dogfood profile.

## Residual: cross-version-upgrade-not-yet-demonstrable
Accepted: no earlier schema exists; revisit for every schema-changing release

There is no earlier published schema from which to upgrade. A later schema-changing release must
add an old-image to new-image preservation demonstration before it can claim forward compatibility.
