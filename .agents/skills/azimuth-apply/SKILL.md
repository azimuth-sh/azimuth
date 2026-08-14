---
name: azimuth-apply
description: Implement an approved Azimuth change, update its current facets, establish honest automated, manual, or operational evidence, and leave it ready for finalization. Use after a proposal is approved or when resuming an active change; do not archive it.
---

# Apply a change

Implement the approved target without letting the proposal become a substitute for source truth.

## Workflow

1. Read every change artifact and run `azimuth change status <id>`. Read affected current model
   packages and the project evidence standard.
2. If `work-packages.md` exists, use `azimuth-coordinate`; otherwise follow dependency order in
   `plan.md`.
3. Implement observable behaviour and mechanisms. Keep product decisions within the approved
   boundary; record necessary departures immediately rather than silently rewriting the proposal.
4. Apply accepted intent deltas to package `spec.md`. Distil only mechanisms that now exist into
   current `design.md`, and only lasting evidence deviations or residuals into
   `verification.md`.
5. Add `Realizes` only where the site establishes part of the named predicate. For every `Covers`
   addition or change, invoke `azimuth-cover` and declare the evidence's actual scope,
   quantification and oracle.
6. Build and run the narrow evidence while iterating, then the affected component and composed
   evidence. Emit every relevant language manifest and run `azimuth check` over their union.
7. When the change adds or alters a surface, run its real enumerator and validate the negative
   path with a temporary representative untagged member. Expect `invariant-breach`, then remove the
   temporary member. Satisfy area realization obligations with honest production relations; do not
   manufacture one evidence item per area.
8. Invoke `azimuth-verify` for every new or stale judgment. Fix dishonest relations and toothless
   evidence rather than editing the verdict to green.
9. Complete plan and work-package statuses, write `outcome.md`, and leave proposal status at
   `implemented` until acceptance is genuinely established.

## Boundaries

- Do not archive; `azimuth-archive` owns the acceptance boundary.
- Do not turn a planned mechanism into current design before its binding exists.
- Do not label ordinary tests as Azimuth evidence for routine claims.
- A passing command is execution evidence, not proof that its tags are honest.
