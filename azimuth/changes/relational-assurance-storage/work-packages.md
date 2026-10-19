# Work packages

## Work package: run-schema
Status: complete
Depends on: none
Owns: services/assurance/server/src/relational/run.rs
Objective: Declare complete explicit schemas for the native Run protocol, all historical Subject variants, launch records, contributions and nullable/optional fields; preserve wire values without generic document storage.
Evidence: Native Run fixtures and contract field coverage with coordinator round-trip tests.

## Work package: account-schema
Status: complete
Depends on: none
Owns: services/assurance/server/src/relational/account.rs, services/assurance/server/src/relational/release.rs
Objective: Declare complete explicit schemas for authority, historical/current reviews, catalogs, full release publications, changes, deployments and observation records using existing strict contracts; no unknown-field fallback.
Evidence: Native authority/review/release fixtures and retained real package round trips through coordinator tests.

## Work package: assurance-domain
Status: complete
Depends on: none
Owns: services/assurance/server/src/lib.rs, services/assurance/server/src/relational/domain.rs, services/assurance/server/tests/relational_domain.rs
Objective: Declare earlier assurance-domain schemas and replace payload persistence in earlier API with shared relational store/reconstruction; integrate fresh SQLx migrations and initialization without mutating historical migration files.
Evidence: Existing domain/lifecycle tests and schema round trips; ordinary CI integration.

## Work package: release-storage
Status: complete
Depends on: none
Owns: services/assurance/server/src/run_ledger/release.rs, services/assurance/server/src/run_ledger/deployment_observation.rs, services/assurance/server/src/run_ledger/model_catalog.rs, services/assurance/server/src/run_ledger/environment.rs
Objective: Replace JSON payload persistence with typed relational store and SQL wire reconstruction in release, catalog and deployment APIs. Preserve atomic immutable histories, authority and idempotency semantics; metadata tables retain scalar columns only.
Evidence: Existing release and environment lifecycle tests with relational schema initialization.
