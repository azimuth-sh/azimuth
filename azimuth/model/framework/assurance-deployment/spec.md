# Spec: framework/assurance-deployment

## Requirement: private-network-containment
Title: Private network containment
Criticality: routine

The private assurance profile SHALL expose its unauthenticated processes only through an explicit
operator-controlled network boundary.

### Scenario: deployment-owns-secrets
- Event: the private assurance profile is resolved
- Required outcome: its database credential is supplied by the deployment
- Additional condition or outcome: the repository supplies no usable default credential

### Scenario: host-bindings-match-private-boundary
- Event: the private assurance profile is resolved
- Required outcome: PostgreSQL has no host-published port
- Additional condition or outcome: application host ports bind only to loopback

### Scenario: containment-is-not-authentication
- Event: an operator evaluates the private assurance profile
- Required outcome: the trusted proxy, tunnel or VPN boundary is explicit
- Additional condition or outcome: application authentication and direct-internet readiness are explicitly denied

## Requirement: assurance-ledger-durability
Title: Assurance ledger durability
Criticality: routine

The private assurance profile SHALL preserve accepted ledger history through its documented
service lifecycle.

### Scenario: history-survives-current-service-recreation
- Context: accepted account and execution history in the assurance ledger
- Event: the composed stack is stopped, started and its current service containers are recreated
- Required outcome: the same accepted history remains externally readable

### Scenario: retained-database-migrates-forward
- Context: the private profile's retained PostgreSQL volume
- Event: the current application starts
- Required outcome: every repository migration is applied automatically in forward order
- Additional condition or outcome: downgrade migration is not a supported operation

## Requirement: selected-image-platforms-build
Title: Selected image platforms build
Criticality: routine

Every assurance image and platform selected by the release contract SHALL produce a buildable OCI
candidate before publication orchestration begins.

### Scenario: complete-selected-image-matrix-builds
- Event: assurance image candidates are qualified
- Required outcome: the image and platform population is derived from the release catalog
- Additional condition or outcome: every image builds for every selected platform
- Additional condition or outcome: no candidate is published by qualification
