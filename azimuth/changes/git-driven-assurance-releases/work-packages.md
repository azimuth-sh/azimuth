# Work packages

## Work package: release-implementation
Status: in-progress
Depends on: none
Owns: contracts, docs, tools/azimuth/src, tools/azimuth/resources, tools/extractors/dotnet, tools/extractors/typescript, tools/extractors/helm, services, .github, scripts
Objective: Implement authorized release handoff, ASS owner acceptance and observations, standalone deployment reconciler and consumer/infra integrations under the same worker. External consumer paths are tracked explicitly in design and coordinator ownership note; no path escapes the canonical package.
Evidence: Native validation, compiler/type checks and static workflow/configuration inspection; tests now authorized and exercised; runtime rollout and owner acceptance remain pending.

## Work package: source-fixtures
Status: complete
Depends on: none
Owns: tools/azimuth/tests/assurance.rs, tools/azimuth/tests/cli.rs, tools/azimuth/tests/decision_scope.rs, tools/azimuth/tests/packages.rs, tools/azimuth/tests/project_validation.rs, tools/azimuth/tests/spec_parse.rs, tools/azimuth/tests/traceability.rs, tools/azimuth/tests/validation.rs, tools/azimuth/tests/verification.rs
Objective: Align stale integration fixtures with current stable Claim identities while preserving their behavioral assertions.
Evidence: Actual CI failures and compilation; normal CI executes the tests.

## Work package: compact-review-fixtures
Status: complete
Depends on: none
Owns: tools/azimuth/tests/compact_assurance_reviews.rs
Objective: Correct current compact review fixtures against the supported review input contract without weakening behavioral assertions.
Evidence: Actual CI failures and compilation; normal CI executes the tests.

## Work package: federation-fixtures
Status: complete
Depends on: none
Owns: tools/azimuth/tests/federation.rs
Objective: Correct federated model fixtures for project-global Claim identities without weakening behavioral assertions.
Evidence: Actual CI failures and compilation; normal CI executes the tests.

## Work package: python-cpp-fixtures
Status: complete
Depends on: none
Owns: tools/extractors/python, tools/extractors/cpp
Objective: Repair Python and C++ extractor fixtures against the current project-global identifier contracts.
Evidence: Run existing focused tests and rehearsals, retain failure diagnostics, then verify normal CI.

## Work package: go-rust-jvm-fixtures
Status: complete
Depends on: none
Owns: tools/extractors/go, tools/extractors/rust, tools/extractors/jvm
Objective: Repair Go, Rust and JVM extractor fixtures against current project-global identifier contracts.
Evidence: Run existing focused tests and rehearsals, retain failure diagnostics, then verify normal CI.

## Work package: experiment-fixtures
Status: complete
Depends on: none
Owns: experiments
Objective: Repair actual downstream experiment failures against current identifier and publication contracts without weakening evidence assertions.
Evidence: Run existing focused tests and rehearsals, retain failure diagnostics, then verify normal CI.
