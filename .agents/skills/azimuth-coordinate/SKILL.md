---
name: azimuth-coordinate
description: Coordinate an approved Azimuth change through dependency-ordered, path-isolated work packages using the coding agent's native sub-agent or team capability when available. Use for changes with work-packages.md, multiple repositories, or independently implementable slices; fall back to sequential execution when delegation is unavailable.
---

# Coordinate work packages

The CLI computes the eligible frontier; the host agent runtime performs delegation. Azimuth does
not pretend that one vendor-neutral binary can spawn every coding agent.

## Workflow

1. Run `azimuth change work-packages <id>`. Stop on an unknown dependency, dependency cycle or
   overlapping ownership error.
2. Keep shared contracts, change artifacts and integration files under coordinator ownership.
   Create a predecessor package for frozen contracts before parallel consumers start.
3. For every eligible package, run
   `azimuth change instructions <id> --package <package-id>` and give that output to a fresh agent.
   Use the runtime's native parallel-agent capability when available. Otherwise execute the same
   packages sequentially without changing their boundaries.
4. Require each worker to report changed files, commands run, evidence results and residuals. A
   worker must not finalize, archive, create another proposal, or edit outside its Owns paths.
5. Review the result against its objective and scope before changing its Status to `complete`.
   Reject unreported cross-package edits even when tests pass.
6. Re-run the CLI to compute the next eligible frontier. Continue until no pending package remains.
7. Integrate shared manifests, run complete checks and composed evidence, and hand the change to
   `azimuth-apply` for current-facet distillation and `azimuth-archive` for acceptance.

## Failure rules

- If an eligible package cannot proceed because a contract is not settled, move that contract into
  a predecessor package; do not let workers negotiate incompatible copies.
- If two packages repeatedly need the same files, the decomposition is false. Merge or redraw
  ownership instead of adding exceptions.
- A local green result in a federated project is incomplete. The coordinator owns complete project
  assembly and exact-revision receipts.
