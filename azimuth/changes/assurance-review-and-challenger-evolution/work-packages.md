# Work packages: assurance-evolution

Direct implementation authorized by Dima through the reviewed assurance-evolution plan. Coordinator retains contracts, documentation and consumer account integration. No workflow replay, commits, publication or deployments.

## Work package: review-workflow
Status: pending
Depends on: none
Owns: azimuth/changes/assurance-review-and-challenger-evolution/deltas, tools/azimuth/src/assurance_review.rs, tools/azimuth/src/review_authoring.rs, tools/azimuth/src/workflow_state.rs, tools/azimuth/resources/workflows, tools/azimuth/resources/skills
Objective: Implement Claim Review and Claim Health native records and aggregate Markdown compilation, replace mandatory old assurance review workflow stages, preserve currentness, correction loops and local-first publication boundaries.
Evidence: Build and inspect native product command outputs with generic synthetic inputs. No test authoring or execution.

## Work package: challenger-execution
Status: pending
Depends on: none
Owns: tools/azimuth/src/account_verification.rs, tools/azimuth/src/account_model.rs, tools/azimuth/src/model.rs, tools/azimuth/src/run.rs, tools/azimuth/src/run_plan.rs, tools/azimuth/src/run_selection.rs, tools/azimuth/src/verification.rs, tools/azimuth/src/fingerprint.rs, tools/extractors, packages
Objective: Implement compact machine Challenger declarations, source implementation markers, scope preparation and adapter integration, exact assessment targets and source/artifact/deployment Subject semantics. Expose bounded mutation and static-analysis execution support without execution or tests.
Evidence: Build and product command inspection, strict mapping and result diagnostics, preserved historical records. No test authoring or execution.

## Work package: assurance-service
Status: pending
Depends on: none
Owns: services/assurance
Objective: Implement persistence/API/UI support for Claim Reviews, Claim Health, Challenger findings and source/artifact/deployment provenance, retaining authentication and immutable history.
Evidence: Service and UI builds and inspection only. No tests, image publication or deployment.
