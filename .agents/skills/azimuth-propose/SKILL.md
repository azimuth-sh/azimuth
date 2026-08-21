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

Strict adapter configuration, the description handshake, complete-model Check planning and bounded
execute/import transport are current. Configuration defaults to `azimuth/adapters.json`, names exact
capability addresses and pins content, description, semantic settings, literal environment and
process limits. A proposal must preserve core authority over the semantic Plan: an adapter only
translates frozen selections or imports exact content-addressed native files.

The current planner emits Checks and `challenges: []`; it does not resolve Challenge Plans,
Qualifications or Claim Judgments. Hand-authored strict launch plans may exercise Challenge
transport without creating model authority. `model.extract` is a declared capability but has no
current execution command. The Run ledger separately owns durable ingest, authorization, retention
and Assurance State. A proposal must not infer those authorities from a valid bundle, invent their
contracts or restore removed alpha-era formats.

When a proposal affects adapters, make the strict
[adapter protocol](../../../azimuth/formats/adapter.md) and
[launch-plan format](../../../azimuth/formats/run-launch-plan.md) explicit. Preserve direct
shell-free invocation, same-stream content staging, exact literal child environment, supported
fresh process-group isolation before spawn, bounded streams and one bounded core exchange whose
deadline covers request writing, concurrent stream draining and core's wait. Validate completely
before atomic output. Core signals the group on every terminal path and cleans members and inherited
pipes while they remain in it. Authorized descendants may escape with `setsid`, `setpgid` or
equivalent, and their termination is not guaranteed. This is not non-escapable descendant
containment, a sandbox, daemon supervision or hostile-code isolation.

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
