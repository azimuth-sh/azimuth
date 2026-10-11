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

Read `workflow next` using the same explicit repository map (or selected `--project`); handoffs carry portable references plus complete local execution locations. Act on `action` and typed `target`, and record results against the returned concrete `id`, basis and current revision. The harness chooses workers and obtains real permission. A local approval label grants no permission itself.

After source support, candidate validation and real same-project registration, supply the strict assurance-target inventory through the eligible inventory action. Core validates the native revision snapshot and expands qualification, execution, applicability and non-routine Claim Judgment targets. Unknown or unavailable inputs remain failed inventories or explicit blockers; file presence creates no process state. Corrective work registering another candidate adds a fresh generation and retains prior tasks as superseded. Preserve every supporting record and use independent reassessment; never invent assurance facts to advance a gate.
