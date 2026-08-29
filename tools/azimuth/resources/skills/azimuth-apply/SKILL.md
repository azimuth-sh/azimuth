---
name: azimuth-apply
description: Implement one approved Azimuth change, update current facets, perform repository-permitted engineering verification and leave the change ready for finalization. Use after proposal approval or when resuming active implementation; do not archive.
---

# Apply an approved change

## Establish the boundary

1. Read the target repository's applicable instructions and the complete approved change.
2. Confirm proposal approval or explicit implementation authorization. Resolve any material contradiction rather than inventing semantics.
3. Inspect the dirty worktree and preserve unrelated user changes.
4. Run `azimuth change show <id>` and the repository's normal `azimuth validate` to establish the baseline.
5. When work packages exist, use `azimuth change work-packages <id>` and `azimuth change brief <id> --package <package>` or invoke `azimuth-coordinate`.

## Implement

Follow the dependency order in `plan.md`. Keep changes inside scope, preserve exact accepted identities and use the target repository's native tooling. Update implementation, migrations, interfaces and documentation together when the approved design requires them. Record actual plan progress without manufacturing verification or rollout facts.

Update current `spec.md` and `design.md` facets only to the state actually established by the implementation. Preserve archived decisions and unrelated accepted intent.

## Verify and stop

Perform only engineering checks permitted by repository instructions. Run `azimuth change check <id>`, the normal `azimuth validate`, and relevant reports or exports. Inspect the final diff and record residuals, unperformed checks and rollout-dependent conditions honestly. Leave the change ready for finalization and ask for the next action. Do not finalize, archive or commit implicitly.
