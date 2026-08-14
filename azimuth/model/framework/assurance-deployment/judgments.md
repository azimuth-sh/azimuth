# Judgments: framework/assurance-deployment

## Claim: deployment-owns-secrets
Verdict: sound
Fingerprint: 405405b5677da3df
Judged: 2026-08-14
Judge: Codex

I inspected both credential consumers in the resolved three-service Compose account and the
qualifier's missing-environment execution. Compose must fail after the qualifier removes the
deployment variable; with the variable present, PostgreSQL and the API connection string must
contain the same qualification-owned marker. A wrong profile with the prior hard-coded password,
a usable interpolation default or mismatched API credential would fail before qualification. The
component and universal tags therefore describe the real Compose resolver and every database
credential input in the bounded profile.

## Claim: host-bindings-match-private-boundary
Verdict: sound
Fingerprint: 1f38fbe8f047f672
Judged: 2026-08-14
Judge: Codex

I inspected every port in the resolved Compose account, the exact three-service population and the
private-network membership checks. PostgreSQL must have no host port; every port on both selected
applications must report host address `127.0.0.1`. The mutation cases add a PostgreSQL port and
replace one application address with `0.0.0.0`, and both fail. A newly added service, unbound
application or non-loopback port also fails the complete-account guard, so the universal component
declaration does not hide an uninspected binding.

## Claim: containment-is-not-authentication
Verdict: sound
Fingerprint: 8b93cf5ae279d8f6
Judged: 2026-08-14
Judge: Codex

I read the operator runbook and the qualifier that normalizes and checks its three separate
security propositions: the trusted proxy, tunnel or VPN boundary, absent application
authentication and absent direct-internet readiness. The unit mutation loop removes each required
statement independently. A wrong runbook that dropped any boundary or allowed the network profile
to imply authentication would fail repository qualification. The evidence establishes explicit
documentation, not the trustworthiness of an operator's external network; that limitation is
recorded as an accepted residual.

## Claim: history-survives-current-service-recreation
Verdict: sound
Fingerprint: 6dbf8f831f3b7efa
Judged: 2026-08-14
Judge: Codex

I inspected the named PostgreSQL volume and the complete lifecycle body. It creates an isolated
Compose project from an empty volume, seeds an account, model snapshot, evidence definition,
qualification, observation and gate decision through HTTP, and canonicalizes the exported public
snapshot. It compares that exact snapshot after stop/start and after forced recreation of
PostgreSQL, API and web. A wrong profile with a missing or miswired volume loses the database when
PostgreSQL is recreated and cannot satisfy the comparison. The e2e and universal tags honestly
range over both documented transitions using real processes and real persistence.

## Claim: retained-database-migrates-forward
Verdict: sound
Fingerprint: 97923628015ab7ce
Judged: 2026-08-14
Judge: Codex

I inspected API startup, the embedded SQLx migration call, the tracked migration population and
the real PostgreSQL oracle. Startup calls `migrate` before binding the HTTP listener. The lifecycle
starts with an empty volume, derives every numeric migration version from the repository, queries
successful rows in `_sqlx_migrations`, and requires the same complete ordered set before and after
service recreation. Removing startup migration makes the empty database unusable; omitting or
failing one tracked migration makes the direct ledger comparison fail. The component-universal
tag is honest for the server and real database; cross-version preservation remains explicitly
residual because no earlier schema exists.

## Claim: complete-selected-image-matrix-builds
Verdict: sound
Fingerprint: e81a08cca13650ae
Judged: 2026-08-14
Judge: Codex

I inspected the two image entries, their catalog-owned build contexts and all four selected
image/platform pairs. The qualifier passes the complete per-image platform set to BuildKit,
exports OCI archives without a tag or registry output, recursively reads nested OCI indexes and
requires their non-attestation platform set to equal the catalog. A wrong Dockerfile that cannot
compile on one architecture or an exporter result missing one manifest fails. The recursive-index
regression rejects the exporter-layout assumption exposed by the first local run, so the component
and universal declaration covers every selected pair rather than the host architecture.
