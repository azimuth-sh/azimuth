# Change: framework-only-change-completion

Status: accepted and complete

Exploration: canonical-alpha-release
Carries decisions: CAR16; enables change-map node C1

## Problem

Local completion currently requires every change to contain a parsed intent addition or
criticality transition. Five transferred framework changes alter protocols, tooling, documentation
or optional infrastructure without changing accepted consumer-domain intent. Inventing an
unrelated intent delta would falsify their records, while accepting every missing delta would hide
accidental or incomplete proposals.

## Outcome

A proposal may declare `Intent delta: none` with a non-empty `Because:` rationale in its metadata.
The declaration makes a framework-only transition explicit and reviewable. Completion accepts that
mode after all existing plan, outcome and hole-free-model gates pass. A missing rationale, an
undeclared missing delta or a proposal that declares both no delta and a supported delta fails.

## Scope

In scope:

- one explicit proposal-metadata declaration for a change with no accepted-intent transition;
- fail-closed parsing for absent rationale, duplicate declaration and contradictory parsed deltas;
- visible `change check` output for the declared completion mode;
- deterministic unit and CLI evidence for acceptance and rejection paths; and
- change-process documentation for when the declaration is honest.

Out of scope:

- weakening plan, outcome, model-finding, finalization-freshness or archive gates;
- inferring framework-only status from prose, paths or an empty `specs/` directory;
- adding replacement, removal or arbitrary model-edit operations to the delta parser;
- revising or archiving the five transferred changes in this enabling change; and
- changing federated project acceptance.

## Affected claims

Add `framework/change-lifecycle#explicit-no-delta-is-reviewable` and
`framework/change-lifecycle#missing-no-delta-declaration-is-rejected` at routine criticality.
These are prerelease workflow obligations with no current external consumer or durable-data
effect. Deterministic tests remain ordinary project evidence outside Azimuth's model, as D20
requires for routine claims.

## Completion conditions

- A zero-delta proposal with one `Intent delta: none` declaration and a non-empty `Because:` has no
  missing-delta completion issue.
- The same proposal still fails for incomplete plan items, missing accepted statuses, missing
  outcome headings or accepted-model findings.
- A zero-delta proposal without the declaration or rationale fails with a precise diagnostic.
- A proposal with both the declaration and a supported intent delta fails as contradictory.
- `azimuth change check` reports the explicit no-delta mode and its rationale.
- The two routine claims are applied to the current framework model.
- Targeted Rust and CLI tests, the five transferred change checks and `./scripts/check.sh` pass
  without reading another repository.
