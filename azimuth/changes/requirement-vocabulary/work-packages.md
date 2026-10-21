# Work packages

Consumer repositories (drim.dev, GitOps, infrastructure) are outside this checkout and are coordinated by the plan, not owned here. The coordinator retains this change directory, `azimuth/installation.json`, `release/artifacts.json` and rollout execution.

## Work package: core
Status: pending
Depends on: none
Owns: tools/azimuth
Objective: Rename the parser, model, Run protocol, fingerprints, review and deployment-status code, CLI, workflow definitions, embedded skills/references/templates and frozen vectors per the design matrix; add the migration edge and successor migration line; replace the retired-vocabulary rejection.
Evidence: `azimuth validate` clean with the built CLI; regenerated frozen vectors derive from the renamed contracts; migration plan/apply round-trips a fixture.

## Work package: contracts-and-model
Status: pending
Depends on: core
Owns: contracts, docs, azimuth/model, azimuth/standards, AGENTS.md, README.md
Objective: Rewrite contracts, glossary and framework prose; migrate the framework's own accepted model syntax and vocabulary with identifiers preserved; keep contracts/history untouched.
Evidence: `azimuth validate` clean; terminology audit lists only justified exceptions.

## Work package: skills
Status: pending
Depends on: core
Owns: .agents, .claude
Objective: Update agent skills to the renamed vocabulary, including the review skill rename.
Evidence: Skill digests match the embedded resources.

## Work package: packages-and-extractors
Status: pending
Depends on: core
Owns: packages, tools/extractors
Objective: Rename annotation parameters and markers, challenge markers, Execution Context protocol constants and manifest keys in every language package and extractor; keep the Python protocol mirror byte-identical to Rust.
Evidence: Per-package builds and tests; cross-language fingerprint vectors equal the Rust vectors.

## Work package: ass-server
Status: pending
Depends on: core
Owns: services/assurance/domain, services/assurance/server, services/assurance/deployment, services/assurance/seed-demo.sh, services/assurance/docker-compose.yml, services/assurance/README.md, services/assurance/Cargo.toml, services/assurance/Cargo.lock
Objective: Delete the unused legacy gate API (`lib.rs` surface, its tables and seed-demo.sh) as explicitly permitted; rename the remaining domain types, routes, relational registry, migrations, triggers and dependency identities; schema version 6 with fresh-only initialization; keep materialized-assessment architecture unchanged.
Evidence: Fresh empty-database initialization; selective invalidation, pending/expired and time-based invalidation tests; Requirement Review and Status parity fixtures.

## Work package: ass-web
Status: pending
Depends on: ass-server
Owns: services/assurance/web
Objective: Rename routes, components, labels, fixtures and development data; Satisfied / Not satisfied / Unresolved and Check Result outcome labels; preserve themes, navigation and freshness handling.
Evidence: Typecheck, production build, status-view fixture checks.

## Work package: collector-and-reconciler
Status: pending
Depends on: core, packages-and-extractors
Owns: services/collector, services/reconciler
Objective: Rename envelope, payload and Execution Context constants; endpoint default `/internal/execution-context`; keep Observation names for runtime facts.
Evidence: Collector and reconciler tests; outbox behavior against a renamed ASS.

## Work package: experiments
Status: pending
Depends on: packages-and-extractors, ass-server
Owns: experiments
Objective: Update the synthetic conformance experiments (test-report-adapter, adapter-capabilities, challenge-planning, mechanism-identities, run-bundles, assurance-service) and their generators to the renamed wire names, regenerating published vectors from the renamed contracts.
Evidence: Each experiment's own check script passes.

## Work package: release-and-ci
Status: pending
Depends on: packages-and-extractors, ass-server, collector-and-reconciler
Owns: release, scripts, .github
Objective: Update release tooling, protocol ids, artifact manifests and publication workflows for the renamed contracts.
Evidence: CI publication of tooling, packages and images from the renamed sources.

## Work package: active-changes
Status: pending
Depends on: core
Owns: azimuth/changes/assurance-review-and-challenger-evolution, azimuth/changes/claim-first-assurance-accounts, azimuth/changes/claim-titles, azimuth/changes/compact-verification-packages, azimuth/changes/deployment-claim-status, azimuth/changes/git-driven-assurance-releases, azimuth/changes/helm-source-emitter, azimuth/changes/materialized-deployment-assessments, azimuth/changes/module-qualified-check-identities, azimuth/changes/private-assurance-service, azimuth/changes/project-validation-and-run-coordination, azimuth/changes/relational-assurance-storage, azimuth/changes/reviewed-intent-transitions, azimuth/changes/runtime-observation-identity
Objective: Migrate active change artifacts so they validate under the renamed syntax; directory names and recorded history text are preserved.
Evidence: `azimuth validate` clean with no archived path touched.
