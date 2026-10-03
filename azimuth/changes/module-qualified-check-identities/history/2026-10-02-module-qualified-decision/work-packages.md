# Work packages: module-qualified-check-identities

## Work package: core-check-locators
Status: complete
Depends on: none
Owns: tools/azimuth/src/diag.rs, tools/azimuth/src/verification.rs, tools/azimuth/src/account_verification.rs, tools/azimuth/src/manifest.rs, tools/azimuth/src/account_support.rs, tools/azimuth/src/model.rs
Objective: enforce module-qualified Check IDs across definitions and support without changing unrelated entity kinds
Evidence: builds and permitted product commands; no tests

## Work package: run-check-locators
Status: in-progress
Depends on: core-check-locators
Owns: tools/azimuth/src/run.rs, tools/azimuth/src/run_plan.rs, tools/azimuth/src/adapter.rs, tools/azimuth/src/adapter_host.rs
Objective: use strict module-qualified Check IDs for current Run request, planning, routing and exchange boundaries
Evidence: builds and permitted product commands; no tests

## Work package: emitter-check-locators
Status: in-progress
Depends on: none
Owns: tools/extractors, packages, services/assurance
Objective: make ecosystem Check markers fail closed on invalid global locators and emit the approved Check identity unchanged
Evidence: ecosystem builds and emitter commands; no tests or test-file changes

## Work package: identity-integration
Status: in-progress
Depends on: core-check-locators, run-check-locators, emitter-check-locators
Owns: contracts, docs, tools/azimuth/resources, tools/azimuth/src/installation.rs, tools/azimuth/src/resources.rs, azimuth/changes/module-qualified-check-identities, experiments
Objective: integrate contracts and migration guidance, author intent, inspect the full coordinated identity change and record unresolved historical/current boundaries
Evidence: build, product commands, diff inspection; no tests or test-file changes
