# Work packages: project-validation-and-run-coordination

## Work package: project-validation
Status: pending
Depends on: none
Owns: tools/azimuth/src/main.rs, tools/azimuth/src/federation.rs, tools/azimuth/src/change.rs, tools/azimuth/src/validation.rs, tools/azimuth/tests/project_validation.rs
Objective: Implement project and workset native accepted/prospective validation and unified change validate with clear structural, evidence and readiness reporting

## Work package: coordinated-runs
Status: pending
Depends on: none
Owns: tools/azimuth/src/run.rs, tools/azimuth/src/run_plan.rs, tools/azimuth/src/adapter.rs, tools/azimuth/src/adapter_host.rs, tools/azimuth/src/run_selection.rs, tools/azimuth/src/account_verification.rs, tools/azimuth/src/model.rs, tools/azimuth/tests/run.rs, tools/azimuth/tests/run_plan.rs, tools/azimuth/tests/adapter_host.rs, tools/azimuth/tests/run_selection.rs, packages/python/azimuth_run
Objective: Implement bounded multi-adapter Runs for one Subject, derived Case and unit selection and scalable eligibility and explicit capability routing without hidden omissions

## Work package: review-inputs
Status: pending
Depends on: none
Owns: tools/azimuth/src/assurance_review.rs, tools/azimuth/tests/compact_assurance_reviews.rs, services/assurance/server/src/run_ledger.rs
Objective: Generate exact independent review inputs without decisions and inspect or repair dependency-sensitive freshness with incremental reviews

## Work package: integration
Status: pending
Depends on: project-validation, coordinated-runs, review-inputs
Owns: contracts, docs, tools/azimuth/resources, tools/azimuth/src/lib.rs, tools/azimuth/tests/compact_verification_packages.rs, tools/azimuth/tests/cli.rs, tools/azimuth/tests/run_cli.rs, tools/azimuth/tests/adapter_cli.rs, tools/azimuth/tests/adapter.rs, experiments, azimuth/changes/project-validation-and-run-coordination, AGENTS.md
Objective: Coordinate interfaces, align contracts and skills, exercise generic and real account commands and record evidence and residuals
