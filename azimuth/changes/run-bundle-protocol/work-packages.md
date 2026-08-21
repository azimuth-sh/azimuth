# Work packages: run-bundle-protocol

## Work package: change-authority
Status: pending
Depends on: none
Owns: docs/decisions.md, azimuth/formats/run-bundle.md, azimuth/changes/run-bundle-protocol
Objective: freeze the provider-neutral Run contract and accepted transition
Evidence: format review, change-account validation and work-package validation

## Work package: run-format-kernel
Status: pending
Depends on: change-authority
Owns: tools/azimuth/src/run.rs, tools/azimuth/tests/run.rs
Objective: parse, fingerprint, reduce and verify strict Run bundles and correction sets
Evidence: complete subject, selection, reduction, reference and history test matrix

## Work package: run-cli-surface
Status: pending
Depends on: run-format-kernel
Owns: tools/azimuth/src/lib.rs, tools/azimuth/src/main.rs, tools/azimuth/tests/run_cli.rs
Objective: expose only service-free Run verification and inspection
Evidence: help, output parity, exit-class and reserved-subcommand tests

## Work package: run-conformance-experiment
Status: pending
Depends on: run-cli-surface
Owns: experiments/run-bundles
Objective: exercise representative Subjects, aggregation, dual-role execution and corrections
Evidence: standalone synthetic conformance gate

## Work package: run-gate-integration
Status: pending
Depends on: run-conformance-experiment
Owns: scripts/check.sh
Objective: include Run conformance before release qualification
Evidence: shell syntax, isolation discovery and canonical root gate

## Work package: run-public-account
Status: pending
Depends on: run-format-kernel, run-cli-surface
Owns: README.md, docs/framework.md, docs/glossary.md, docs/change-process.md, docs/assurance-extensions.md, azimuth/README.md, tools/azimuth/README.md
Objective: make the implemented Run exchange and remaining deferred boundaries public
Evidence: terminology, command, link, 100-column and prohibited-name audits

## Work package: run-agent-guidance
Status: pending
Depends on: run-cli-surface
Owns: AGENTS.md, .agents/skills/azimuth-propose/SKILL.md, .agents/skills/azimuth-apply/SKILL.md, .agents/skills/azimuth-archive/SKILL.md
Objective: teach current Run verification without inventing adapter or ledger workflows
Evidence: frontmatter, command, link and retired-deferral audits

## Work package: run-intent-cutover
Status: pending
Depends on: change-authority
Owns: azimuth/model/framework/run-bundle-protocol/spec.md
Objective: apply the five routine Run-protocol requirements without verification facets
Evidence: current-model parse, all-routine audit and change status
