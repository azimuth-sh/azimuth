---
name: azimuth-propose
description: Create or revise one bounded Azimuth change proposal from a clear request or an approved exploration. Use when defining target intent, scope, criticality, solution decisions, verification obligations, work packages, or completion conditions before implementation.
---

# Propose one change

Create the smallest semantic transition that can be reviewed and accepted independently.

## Workflow

1. Read `AGENTS.md`, `azimuth/changes/README.md`, affected model packages and any originating
   exploration. In a federated project, locate the singular change authority first.
2. Check active changes with `azimuth change list`. Do not create a competing proposal for an id
   already owned elsewhere.
3. Run `azimuth change create <id> --title <title>` to obtain the lightweight shape.
4. Write the problem, outcome, in/out scope, affected claims and completion conditions. If an
   exploration supplied the direction, record `Exploration:` and the carried decision ids.
5. Add intent deltas only where observable obligations change. Assign criticality from consequence,
   not implementation size.
6. Add `design.md` only when alternatives, boundaries, failure modes or migration order make a
   solution decision reviewable. Add `verification.md` only for deviations, non-test evidence,
   operational evidence or residuals not derived from the project standard.
7. For a site-domain invariant, identify the semantic population before implementation. Reuse a
   declared surface when its membership is exact; otherwise propose area-mount enumerator
   contributions and name what could remain outside them. For an ordinary cross-area claim, add
   area realization obligations only when accepted architecture requires participation there.
   Do not invent roles or mirror the areas into test obligations.
8. If independent execution is useful, write `work-packages.md`. Each package declares Status,
   Depends on, non-overlapping Owns paths, Objective and Evidence. Coordinator-owned shared
   contracts must be a predecessor rather than jointly owned.
9. Run `azimuth change check <id>` and `azimuth change work-packages <id>` when applicable. Resolve
   parser and DAG errors before presenting the proposal.
10. Present the proposal and ask for approval. Do not implement as part of this skill unless the
   user's request already explicitly authorized both proposal and implementation.

## Routine path

A routine change normally needs only `proposal.md`, one intent delta and `plan.md`. Do not add
design, verification, tags or judgments merely because the templates permit them.

## Work-package format

```markdown
# Work packages: <change-id>

## Work package: <id>
Status: pending
Depends on: none
Owns: path/one, path/two
Objective: one bounded result
Evidence: exact commands or evidence obligation
```
