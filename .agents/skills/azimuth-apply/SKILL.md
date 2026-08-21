---
name: azimuth-apply
description: >-
  Implement an approved Azimuth change, update current facets, run honest engineering checks and
  leave the change ready for finalization. Use after approval or when resuming an active change;
  do not archive it.
---

# Apply a change

Implement the approved target without treating proposal prose as source truth.

## Workflow

1. Read every change artifact, affected current model package, D43–D45 and repository guidance.
   Run `azimuth change status <id>`.
2. If `work-packages.md` exists, validate and coordinate it. Otherwise follow dependency order in
   the change's `plan.md`.
3. Implement observable behavior and mechanisms within the approved boundary. Record necessary
   departures instead of silently changing the proposal.
4. Apply accepted intent deltas to package `spec.md`. Distil only mechanisms that now exist into
   current `design.md`.
5. Keep every current framework Claim routine. Do not add a package `verification.md`, Check,
   Evidence Binding or Qualification merely to describe ordinary tests.
6. Add Realizes only where required and where the production site genuinely establishes part of a
   named case-level Claim. An unmarked native test remains ordinary engineering work.
7. Build and run focused tests while iterating, then affected component and composed suites. Emit
   every relevant language manifest and run `azimuth validate` over their union.
8. When the change alters an enumerated surface, run its real enumerator and validate the negative
   path with a temporary representative untagged member. Remove the temporary member afterward.
9. Run `azimuth report traceability` when relationships change and inspect version 2 export when a
   public graph shape changes.
10. Complete plan and work-package statuses, write `outcome.md`, and leave the proposal at
    `implemented` until acceptance is genuinely established.

## Alpha-2 boundaries

- Azimuth is an evidence control plane; repository declarations own intent and reviewed meaning.
- Future non-routine verification follows Check → Evidence Binding → Qualification.
- Source may implement a Check but never owns its Claim relationship, form or context.
- Challengers and Challenge Plans search for objections; a clean challenge is not product evidence.
- Run, adapter, normalized-bundle and ledger contracts are deferred to dependent changes.
- The D42 service wire remains isolated, and no Assurance Service export command exists.
- Use `azimuth validate`, `azimuth report traceability` and `azimuth export` for the current model.
- Do not archive; `azimuth-archive` owns the acceptance boundary.
