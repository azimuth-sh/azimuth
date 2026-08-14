# Change: private-assurance-deployment

Status: accepted and complete

Exploration: canonical-alpha-release
Carries decisions: CAR9, CAR14, CAR15

## Problem

The assurance service is a local evaluation stack. Its Compose definition hard-codes the database
credential, publishes PostgreSQL on every host interface and publishes both applications without
stating a trusted ingress boundary. The named volume and startup migration establish useful
mechanisms, but no evidence shows that accepted ledger history survives service-container
recreation. The selected image catalog names Linux AMD64 and ARM64 without building both variants.

Deploying that stack for private dogfood would blur three different propositions: network
containment, application authentication and public-internet hardening. Only the first is required
for this change. Treating the other two as implied would turn an unstated operator assumption into
a security claim.

## Outcome

The assurance service has one documented private single-team deployment profile. Deployment-owned
secrets are required at configuration time, PostgreSQL has no host binding, and application ports
bind only to loopback for a trusted reverse proxy, SSH tunnel or VPN boundary. The runbook states
that the service has no application authentication and is not suitable for direct internet
exposure.

A composed-stack lifecycle demonstrates that accepted ledger history remains readable after stop,
start and current-service-container recreation. Startup applies the repository's forward SQLx
migrations to the retained database. A later schema-changing release still owes old-image to
new-image upgrade evidence; this first schema cannot manufacture that evidence.

Both assurance images build for every platform selected by the release catalog. This change
qualifies buildability; it does not publish an image index.

## Scope

In scope:

- a private single-team Compose profile with deployment-supplied database credentials;
- loopback-only API and web bindings and no PostgreSQL host binding;
- an explicit trusted proxy, tunnel or VPN operator boundary;
- an explicit denial of application authentication and direct-internet readiness;
- persistent PostgreSQL storage and automatic forward migration on startup;
- lifecycle evidence for retained ledger data across stop, start and service recreation;
- Linux AMD64 and ARM64 builds for both selected assurance images; and
- deterministic configuration and failure-case checks in the repository gate.

Out of scope:

- application users, sessions, authorization or tenant isolation;
- TLS termination, public ingress, rate limiting, abuse protection or internet hardening;
- secret generation, rotation or an external secret manager;
- PostgreSQL backup, restore, high availability or disaster recovery;
- downgrade migrations or compatibility with a future schema not present in this release;
- publishing, signing or attesting image manifests; and
- referrals or any Drim-owned domain functionality.

## Affected claims

Add `framework/assurance-deployment#private-network-containment` at standard criticality. A broken
binding or embedded credential invalidates the documented private deployment boundary, but the
profile is not an authorization control and has no existing consumer.

Add `framework/assurance-deployment#assurance-ledger-durability` at critical criticality. Accepted
ledger history is the assurance record; losing it during a documented lifecycle is irreversible
data loss. Its truth depends on real PostgreSQL persistence and process composition, so the plan
raises verification scope to end to end.

Add `framework/assurance-deployment#selected-image-platforms-build` at standard criticality. A
missing architecture blocks an intended operator platform before publication but does not corrupt
durable data.

## Completion conditions

- Compose interpolation fails when the deployment credential is absent, and the resolved
  configuration contains no repository-owned default database password.
- PostgreSQL has no host-published port; API and web host bindings resolve only to loopback; and
  every selected application still communicates on the private Compose network.
- The operator runbook names the trusted proxy, tunnel or VPN boundary and states separately that
  the profile supplies neither application authentication nor direct-internet hardening.
- A real composed-stack check registers an account and accepted execution data, records their
  externally observable representation, stops and starts the stack, recreates the current API and
  service containers, and observes the same representation afterward.
- The same lifecycle starts from an empty named volume, applies the current migration set
  automatically and leaves its PostgreSQL volume in place. Documentation makes downgrades
  unsupported and makes old-to-new upgrade evidence a completion condition for a later
  schema-changing release.
- Both catalog-selected Dockerfiles build OCI candidates for Linux AMD64 and ARM64, with the image
  and platform population derived from the release catalog.
- Synthetic configuration violations fail, the complete repository gate passes, and the critical
  durability claim records a residual account.
- No command in this change publishes an image or accesses a public registry with release
  credentials.
