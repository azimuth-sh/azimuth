---
name: azimuth-propose
description: >-
  Create or revise one bounded Azimuth change proposal from a clear request or approved
  exploration. Use to define intent, scope, routine criticality, solution decisions, work packages
  and completion conditions before implementation.
---

# Propose one change

Create the smallest semantic transition that can be reviewed and accepted independently.

## Workflow

1. Read `AGENTS.md`, `azimuth/changes/README.md`, affected model packages and any originating
   exploration. In a federated project, locate singular change authority first.
2. Check active changes with `azimuth change list`. Do not create competing authority for one id.
3. Run `azimuth change create <id> --title <title>` to obtain the lightweight shape.
4. Write the problem, outcome, in/out scope, affected Claim ids and completion conditions. Record
   originating exploration decisions when applicable.
5. Add intent deltas only where observable obligations change. Keep every current framework Claim
   routine during the fast-moving alpha.
6. Add change `design.md` only when alternatives, boundaries, failure modes or migration order make
   a solution decision reviewable. Do not add package verification declarations for routine Claims.
7. For a site-domain invariant, identify the semantic population before implementation. Reuse an
   exact declared surface or propose area-mount enumerator contributions.
8. If independent execution is useful, write `work-packages.md`. Each package declares status,
   dependencies, non-overlapping owned paths, objective and ordinary engineering checks.
9. Run `azimuth change check <id>` and `azimuth change work-packages <id>` when applicable. Resolve
   parser and dependency errors before presenting the proposal.
10. Present the proposal and ask for approval. Do not implement unless the request explicitly
    authorizes both proposal and implementation.

## Model boundary

The accepted future non-routine graph is Check → Evidence Binding → Qualification, with sparse
many-to-many Check/Claim relationships and semantic Challenge Plan selectors. The strict
[Run bundle format](../../../azimuth/formats/run-bundle.md) is current:
`azimuth run verify --bundle <file>...` checks standalone protocol and correction consistency, and
`azimuth run inspect --bundle <file>...` presents a deterministic account with current model
authority and Assurance State explicitly unresolved.

Adapter configuration, plan generation, provider execute/import transport and native translation
remain deferred. The Run ledger separately owns durable ingest, authorization and Assurance
State. A proposal must not infer those authorities from a valid bundle, invent their contracts or
restore removed alpha-era formats.

## Routine path

A routine change normally needs `proposal.md`, any actual intent delta and `plan.md`. It uses
ordinary engineering tests and adds no Check, binding or Qualification.

## Work-package format

```markdown
# Work packages: <change-id>

## Work package: <id>
Status: pending
Depends on: none
Owns: path/one, path/two
Objective: one bounded result
Evidence: exact engineering commands
```
