# Spec: framework/assurance-deployment

## Claim: private-network-containment
Criticality: routine

The private assurance profile SHALL expose its unauthenticated processes only through an explicit
operator-controlled network boundary.

### Case: deployment-owns-secrets
WHEN the private assurance profile is resolved
THEN its database credential is supplied by the deployment
AND the repository supplies no usable default credential

### Case: host-bindings-match-private-boundary
WHEN the private assurance profile is resolved
THEN PostgreSQL has no host-published port
AND application host ports bind only to loopback

### Case: containment-is-not-authentication
WHEN an operator evaluates the private assurance profile
THEN the trusted proxy, tunnel or VPN boundary is explicit
AND application authentication and direct-internet readiness are explicitly denied

## Claim: assurance-ledger-durability
Criticality: routine

The private assurance profile SHALL preserve accepted ledger history through its documented
service lifecycle.

### Case: history-survives-current-service-recreation
GIVEN accepted account and execution history in the assurance ledger
WHEN the composed stack is stopped, started and its current service containers are recreated
THEN the same accepted history remains externally readable

### Case: retained-database-migrates-forward
GIVEN the private profile's retained PostgreSQL volume
WHEN the current application starts
THEN every repository migration is applied automatically in forward order
AND downgrade migration is not a supported operation

## Claim: selected-image-platforms-build
Criticality: routine

Every assurance image and platform selected by the release contract SHALL produce a buildable OCI
candidate before publication orchestration begins.

### Case: complete-selected-image-matrix-builds
WHEN assurance image candidates are qualified
THEN the image and platform population is derived from the release catalog
AND every image builds for every selected platform
AND no candidate is published by qualification
