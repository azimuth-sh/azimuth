# Work packages: validation-command-surface

## Work package: semantic-authority
Status: complete
Depends on: none
Owns: docs/decisions.md, azimuth/model/framework/validation-command-surface
Objective: freeze command, Finding and traceability semantics and apply the routine current intent
Evidence: change-account validation and current model parse

## Work package: validation-engine
Status: complete
Depends on: semantic-authority
Owns: tools/azimuth/src/check.rs, tools/azimuth/src/validation.rs, tools/azimuth/src/model.rs, tools/azimuth/src/assurance.rs, tools/azimuth/src/change.rs, tools/azimuth/src/federation.rs, tools/azimuth/tests/checks.rs, tools/azimuth/tests/validation.rs, tools/azimuth/tests/designs.rs, tools/azimuth/tests/plans.rs, tools/azimuth/tests/assurance.rs, tools/azimuth/tests/federation.rs
Objective: replace Check and Hole APIs with exhaustive categorized Finding validation
Evidence: affected Rust tests and export-shape assertions

## Work package: traceability-view
Status: complete
Depends on: semantic-authority
Owns: tools/azimuth/src/traceability.rs, tools/azimuth/tests/traceability.rs
Objective: implement a deterministic pure Claim-and-realization projection
Evidence: traceability ordering, selection and no-authority tests

## Work package: command-router
Status: complete
Depends on: validation-engine, traceability-view
Owns: tools/azimuth/src/lib.rs, tools/azimuth/src/main.rs, tools/azimuth/tests/cli.rs, tools/azimuth/Cargo.toml
Objective: install validate and report traceability while deleting top-level compatibility parsing
Evidence: CLI help, exit-code, rejection, initialization and report integration tests

## Work package: active-guidance
Status: complete
Depends on: command-router
Owns: AGENTS.md, README.md, tools/azimuth/README.md, docs/framework.md, docs/glossary.md, azimuth/standards/judgment.md, services/assurance/README.md, tools/extractors/README.md
Objective: move current public guidance to validation and Finding terminology
Evidence: active-command and terminology audits plus link checks

## Work package: executed-surfaces
Status: complete
Depends on: command-router
Owns: .agents/skills/azimuth-apply/SKILL.md, .agents/skills/azimuth-archive/SKILL.md, .agents/skills/azimuth-cover/SKILL.md, release/check.sh, experiments/polyglot/check.sh, experiments/assurance-extensions/check.sh
Objective: migrate executable workflows and agent instructions without editing immutable accounts
Evidence: shell syntax, skill audits and executed repository gates
