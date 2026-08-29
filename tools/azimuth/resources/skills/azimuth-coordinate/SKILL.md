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
