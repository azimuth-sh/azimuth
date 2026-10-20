---
name: azimuth-coordinate
description: Coordinate an approved Azimuth change through dependency-ordered, path-isolated work packages using the coding agent's delegation capability when available. Use for changes with validated work packages, multiple repositories or independently implementable slices.
---

# Coordinate work packages

1. Read the target repository's instructions and complete approved change.
2. Run `azimuth change work-packages <id>`. Stop on unknown dependencies, cycles, overlapping or escaping owned paths, invalid status or missing objectives.
3. Keep shared contracts, proposal state, current-facet integration, outcome and archival paths under coordinator ownership.
4. Select only eligible packages. Generate each worker handoff with `azimuth change brief <id> --package <package>`.
5. Delegate only when the available agent runtime supports it. Otherwise execute packages sequentially using the same ownership boundary.
6. Require each worker to edit only declared paths and report files, permitted checks, results and residuals. Workers never finalize or archive.
7. Inspect returned diffs before marking package progress. Resolve integration work centrally and re-run package eligibility after every completion.
8. When all packages are integrated, return to `azimuth-apply` for current-facet updates and final readiness checks.

## Standard change workflow selection

Select `azimuth workflow init --workflow change-development --inputs <file> --instance <file> --repositories <file>` for ordinary change coordination. Inspect `azimuth workflow definition --workflow change-development` and `azimuth reference show workflow` for the exact input and inventory contracts. Supply explicit project/change identities, original request, optional referenced task instructions and declared module inventory; do not manually compose a process DAG or invent context-specific action names. Instruction overrides can direct retained-artifact copying while preserving approval and independent review gates.

Read `workflow next` using the same explicit repository map (or selected `--project`); handoffs carry portable references plus complete local execution locations. Act on `action` and typed `target`, and author one compact result (`work`, optional `target`, `outcome`, optional `report`, `outputs`). Pass the captured handoff, actual actor and current revision to `workflow update`; Azimuth derives identities, fingerprints and dependencies. Reuse the result as the report instead of writing duplicate wrapper files. The harness chooses workers and obtains real permission. A local approval label grants no permission itself.

After structural validation, prepare the complete local candidate authority and snapshot. Azimuth derives targets automatically. Independently review each non-routine Claim definition, Cases, realization and verification through Claim Review; no execution Subject belongs to that review. Execute Checks against explicitly selected exact Subjects, and run declared machine Challengers against exact current assessments with resolved source scope. Publish actual Runs and effective context captures. The deployment assessor computes Claim Status from current Claim Review, exact release lineage, observed deployment composition, applicable evidence and fresh monitoring; do not author a separate positive status record. Different Subjects are joined only through those validated applicability relations. Keep local iterations local; publish the relied-on review and execution package from CI or another authorized boundary. Preserve adverse facts and reassessments; clean Challenge Results never establish support.
