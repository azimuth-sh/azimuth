# Work packages: adapter-capability-protocol

## Work package: protocol-authority
Status: complete
Depends on: none
Owns: docs/decisions.md, azimuth/formats/adapter.md, azimuth/formats/run-bundle.md, azimuth/formats/run-launch-plan.md
Objective: freeze adapter configuration, capability, launch, transport and provenance contracts
Evidence: format review, strict examples and canonical-fingerprint vectors

## Work package: run-component-seam
Status: complete
Depends on: protocol-authority
Owns: tools/azimuth/src/run.rs, tools/azimuth/tests/run.rs
Objective: expose reusable D46 construction and provenance validation without weakening verification
Evidence: component-construction, provenance-extension and regression tests

## Work package: adapter-contract-kernel
Status: complete
Depends on: protocol-authority
Owns: tools/azimuth/src/adapter.rs, tools/azimuth/tests/adapter.rs
Objective: parse strict configuration and derive descriptor, configuration and capability identity
Evidence: config allowlist, drift, address, class, path and fingerprint tests

## Work package: semantic-run-planner
Status: complete
Depends on: run-component-seam, adapter-contract-kernel
Owns: tools/azimuth/src/run_plan.rs, tools/azimuth/src/fingerprint.rs, tools/azimuth/src/change.rs, tools/azimuth/tests/run_plan.rs
Objective: resolve complete-model Checks and bind canonical Check-only launch plans
Evidence: complete-model, implementation-closure, route-substitution and empty-challenge tests

## Work package: bounded-adapter-host
Status: pending
Depends on: run-component-seam, adapter-contract-kernel, semantic-run-planner
Owns: tools/azimuth/src/adapter_host.rs, tools/azimuth/tests/adapter_host.rs
Objective: invoke bounded execute/import processes and validate complete returned bundles atomically
Evidence: handshake, staging cleanup, bounds, import digest, predecessor terminal and no-retry tests

## Work package: adapter-module-integration
Status: pending
Depends on: bounded-adapter-host
Owns: tools/azimuth/src/lib.rs
Objective: export the adapter, planner and host modules before public command routing
Evidence: module compilation and complete Rust library tests

## Work package: adapter-cli-surface
Status: pending
Depends on: adapter-module-integration
Owns: tools/azimuth/src/main.rs, tools/azimuth/tests/adapter_cli.rs, tools/azimuth/tests/run_cli.rs
Objective: expose adapter verify and Run plan, execute and import with exact exit classes
Evidence: help, option, atomic-output, exit-class and absent-ingest tests

## Work package: adapter-conformance-experiment
Status: pending
Depends on: adapter-cli-surface
Owns: experiments/adapter-capabilities
Objective: exercise executing and report-importing synthetic adapters through one conformance suite
Evidence: execute, import, correction, drift, substitution, negative-fact and dual-role cases

## Work package: adapter-gate-integration
Status: pending
Depends on: adapter-conformance-experiment
Owns: scripts/check.sh
Objective: include adapter conformance before release qualification
Evidence: shell syntax, release-isolation discovery and canonical root gate

## Work package: current-intent-transition
Status: pending
Depends on: protocol-authority, adapter-cli-surface
Owns: azimuth/model/framework/adapter-capability-protocol, azimuth/model/framework/run-bundle-protocol/spec.md
Objective: apply routine adapter intent and narrow the superseded Run command boundary to ingest
Evidence: current-model parse, all-routine audit and absence of the superseded current Claim

## Work package: adapter-public-account
Status: pending
Depends on: adapter-cli-surface, adapter-conformance-experiment, current-intent-transition
Owns: README.md, docs/framework.md, docs/glossary.md, docs/change-process.md, docs/assurance-extensions.md, azimuth/README.md, tools/azimuth/README.md
Objective: document the implemented adapter journey and its challenge-planning and ledger boundaries
Evidence: command, terminology, link, 100-column and prohibited-name audits

## Work package: adapter-agent-guidance
Status: pending
Depends on: adapter-cli-surface, current-intent-transition
Owns: AGENTS.md, .agents/skills/azimuth-propose/SKILL.md, .agents/skills/azimuth-apply/SKILL.md, .agents/skills/azimuth-archive/SKILL.md
Objective: teach explicit planning and invocation without inventing decision or state authority
Evidence: frontmatter, command, boundary and stale-guidance audits

## Work package: integration-audit
Status: pending
Depends on: adapter-gate-integration, adapter-public-account, adapter-agent-guidance
Owns: azimuth/changes/adapter-capability-protocol
Objective: reconcile the complete change, record departures and prepare coordinator-only acceptance
Evidence: change check, work-package graph, full root gate and composed-model audit
