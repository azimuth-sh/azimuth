# Work packages: traceability-challenge-planning

## Work package: protocol-authority
Status: complete
Depends on: none
Owns: docs/decisions.md, azimuth/formats/verification.md, azimuth/formats/run-bundle.md, azimuth/formats/run-launch-plan.md, azimuth/standards/verification.md, azimuth/changes/traceability-challenge-planning/design.md
Objective: freeze Claim Judgment, decision-policy, scheduling, semantic-scope and launch-input contracts
Evidence: strict examples, canonical fingerprint vectors, 100-column and contradiction audits

## Work package: decision-model-kernel
Status: complete
Depends on: protocol-authority
Owns: tools/azimuth/src/verification.rs, tools/azimuth/tests/verification.rs, tools/azimuth/tests/spec_parse.rs
Objective: parse and validate strict current Claim Judgment, policy and schedule declarations
Evidence: strict grammar, declaration cardinality, canonical vectors and old-format rejection tests

## Work package: challenge-resolution-projection
Status: complete
Depends on: decision-model-kernel
Owns: tools/azimuth/src/validation.rs, tools/azimuth/src/traceability.rs, tools/azimuth/tests/validation.rs, tools/azimuth/tests/traceability.rs
Objective: resolve all candidate dispositions and project exact decision-impact edges
Evidence: seven selectors, disposition visibility, domain gates, deduplication and impact tests

## Work package: decision-scope-integration
Status: pending
Depends on: challenge-resolution-projection
Owns: tools/azimuth/src/fingerprint.rs, tools/azimuth/src/model.rs, tools/azimuth/src/lib.rs, tools/azimuth/tests/decision_scope.rs
Objective: expose canonical scope components, resolution export and atomic selected-view closure
Evidence: export parity, adverse-selector retention, exact scope records and selection-closure tests

## Work package: challenge-run-protocol
Status: complete
Depends on: protocol-authority
Owns: tools/azimuth/src/run.rs, tools/azimuth/tests/run.rs
Objective: bind scheduling lane, selector anchors and semantic inputs into Challenge selections
Evidence: parser, canonical fingerprint, reduction, partial omission and mismatch tests

## Work package: semantic-challenge-planner
Status: pending
Depends on: challenge-resolution-projection, challenge-run-protocol, decision-scope-integration
Owns: tools/azimuth/src/run_plan.rs, tools/azimuth/tests/run_plan.rs
Objective: expand requested Challenge Plans and derive exact capability routes and launch inputs
Evidence: complete-model, context, form, cap, dedupe, source-input and mixed-plan tests

## Work package: challenge-cli-cutover
Status: pending
Depends on: semantic-challenge-planner
Owns: tools/azimuth/src/main.rs, tools/azimuth/tests/adapter_cli.rs, tools/azimuth/tests/run_cli.rs
Objective: accept strict Challenge requests without weakening command, exit or atomic-output contracts
Evidence: help, schema, Check-only, Challenge-only, mixed, exit-class and output-safety tests

## Work package: challenge-conformance-experiment
Status: pending
Depends on: challenge-cli-cutover
Owns: experiments/challenge-planning
Objective: exercise mutation, dual-role fault and broad-static-analysis selection through public commands
Evidence: selected scope, clean, findings, inconclusive, partial, drift and zero-resolution cases

## Work package: existing-challenge-fixture-migration
Status: pending
Depends on: semantic-challenge-planner, challenge-run-protocol
Owns: tools/azimuth/tests/adapter_host.rs, experiments/run-bundles, experiments/adapter-capabilities
Objective: migrate strict hand-authored Challenge selections and routes to the current protocol shape
Evidence: adapter-host, Run-bundle and adapter-capability conformance gates

## Work package: challenge-gate-integration
Status: pending
Depends on: challenge-conformance-experiment, existing-challenge-fixture-migration
Owns: scripts/check.sh
Objective: add Challenge-planning conformance before service and release qualification
Evidence: shell syntax, isolation discovery and canonical root gate

## Work package: current-intent-transition
Status: pending
Depends on: protocol-authority, challenge-cli-cutover
Owns: azimuth/model/framework/traceability-challenge-planning, azimuth/model/framework/adapter-capability-protocol/spec.md, azimuth/changes/traceability-challenge-planning/specs/framework-traceability-challenge-planning.md
Objective: apply routine intent and replace obsolete Check-only Challenge-planning deferral
Evidence: exact delta projection, all-routine audit and composed validation

## Work package: challenge-public-account
Status: pending
Depends on: challenge-conformance-experiment, current-intent-transition
Owns: README.md, docs/framework.md, docs/glossary.md, docs/change-process.md, docs/assurance-extensions.md, azimuth/README.md, tools/azimuth/README.md
Objective: document current decision authoring, selection, scope, outcomes and ledger boundary
Evidence: command, terminology, link, 100-column and prohibited-name audits

## Work package: challenge-agent-guidance
Status: pending
Depends on: challenge-cli-cutover, current-intent-transition
Owns: AGENTS.md, .agents/skills/azimuth-propose/SKILL.md, .agents/skills/azimuth-apply/SKILL.md, .agents/skills/azimuth-archive/SKILL.md
Objective: teach Challenge planning without inventing state, cache or provider authority
Evidence: frontmatter, links, command, boundary and stale-guidance audits

## Work package: integration-audit
Status: pending
Depends on: challenge-gate-integration, challenge-public-account, challenge-agent-guidance
Owns: azimuth/changes/traceability-challenge-planning/plan.md, azimuth/changes/traceability-challenge-planning/work-packages.md, azimuth/changes/traceability-challenge-planning/proposal.md, azimuth/changes/traceability-challenge-planning/outcome.md, azimuth/changes/traceability-challenge-planning/finalization.json
Objective: reconcile the complete change, record departures and prepare coordinator-only acceptance
Evidence: change check, work-package graph, full root gate and composed-model audit
