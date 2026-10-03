# Work packages: module-qualified-check-identities

## Work package: entity-boundary
Status: complete
Depends on: none
Owns: tools/azimuth/src/diag.rs, contracts/entity-identity.md
Objective: freeze stable entity syntax, typed project context, mutable relationships and source API before parallel consumers
Evidence: reviewed shared helper and contract; isolated parser cargo check passed; no tests

## Work package: entity-parsers
Status: complete
Depends on: entity-boundary
Owns: tools/azimuth/src/spec.rs, tools/azimuth/src/design.rs, tools/azimuth/src/account_design.rs, tools/azimuth/src/verification.rs, tools/azimuth/src/account_verification.rs, tools/azimuth/src/manifest.rs, tools/azimuth/src/account_support.rs
Objective: parse stable project entity IDs and explicit typed relationships without module-derived identity
Evidence: builds and permitted product commands; no tests

## Work package: entity-model
Status: complete
Depends on: entity-boundary
Owns: tools/azimuth/src/model.rs, tools/azimuth/src/lib.rs, tools/azimuth/src/account_model.rs, tools/azimuth/src/validation.rs, tools/azimuth/src/assurance.rs, tools/azimuth/src/traceability.rs, tools/azimuth/src/fingerprint.rs, tools/azimuth/src/workspace.rs, tools/azimuth/src/federation.rs, tools/azimuth/src/change.rs, tools/azimuth/src/intent.rs, tools/azimuth/src/account_delta.rs, tools/azimuth/src/main.rs
Objective: assemble project-wide typed identity indexes and mutable membership/Case-parent relationships throughout current account and assurance operations
Evidence: builds and synthetic product relocation commands; no tests

## Work package: entity-emitters
Status: complete
Depends on: entity-boundary
Owns: tools/extractors, packages, services/assurance
Objective: source APIs use one stable entity ID and emit module-free entity links, preserving semantic source identity and fingerprints
Evidence: all seven ecosystems plus Helm positive/rejection product commands and available source builds passed; no tests or test-file changes

## Work package: entity-runs
Status: complete
Depends on: entity-boundary
Owns: tools/azimuth/src/run.rs, tools/azimuth/src/run_plan.rs, tools/azimuth/src/adapter.rs, tools/azimuth/src/adapter_host.rs
Objective: resolve stable project entity references in planning/exchange and restore taskintroduced protocol versions only
Evidence: builds and permitted product commands; no tests

## Work package: identity-integration
Status: complete
Depends on: entity-parsers, entity-model, entity-emitters, entity-runs
Owns: contracts/account-delta.md, contracts/account-design.md, contracts/account-model.md, contracts/account-support.md, contracts/account-verification.md, contracts/adapter.md, contracts/design.md, contracts/execution-receipt.md, contracts/export.md, contracts/findings.md, contracts/installation.md, contracts/intent-delta.md, contracts/manifest.md, contracts/markers.md, contracts/migration-plan.md, contracts/project-catalog.md, contracts/project-reference.md, contracts/project-snapshot.md, contracts/run-bundle.md, contracts/run-inspection.md, contracts/run-launch-plan.md, contracts/spec.md, contracts/standards.md, contracts/verification.md, contracts/workset.md, contracts/workspace.md, docs, tools/azimuth/resources, tools/azimuth/src/installation.rs, tools/azimuth/src/resources.rs, azimuth/changes/module-qualified-check-identities, azimuth/model, experiments
Objective: migrate current declaration-based accounts, document stable IDs and contextual locators and inspect the complete coordinated transition without modifying historical facts
Evidence: product commands and diff inspection; no tests or test-file changes
