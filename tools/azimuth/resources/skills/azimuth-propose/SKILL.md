---
name: azimuth-propose
description: Create or revise one bounded, approval-ready Azimuth change from a clear request or approved exploration. Use to establish singular authority, author current intent deltas, decide solution boundaries and validate a comprehensive proposal before implementation.
---

# Propose one change

Create the smallest transition that can be reviewed, implemented and accepted independently. Stop after explicit approval of the actual proposal files unless implementation is separately authorized.

## Establish readiness

1. Read the target repository's applicable instructions and change guidance.
2. Confirm the request is sufficiently decided. Require `Status: approved` when carrying an exploration and cite only decisions assigned to this change.
3. Locate singular change authority in a federated project.
4. Inspect current accepted intent, relevant archived decisions, implementation, data, interfaces and permitted engineering checks.
5. Run the repository's normal `azimuth validate` invocation and record pre-existing Findings honestly.

## Author

1. Run `azimuth change list`, choose a stable lower-kebab id and run `azimuth change create <id> --title "<title>"`.
2. Run `azimuth reference show proposal`, `azimuth reference show intent-delta` and any other reference needed for the artifacts being authored.
3. Define the present problem, observable outcome, in-scope and excluded work, exact affected Claim identities, originating exploration decisions and inspectable completion conditions.
4. Add intent deltas only for observable obligations. Do not invent operations omitted by the installed reference.
5. Add `design.md` when implementation would otherwise decide identity, authority, data ownership, compatibility, migration, failure, security, operational or rollback semantics.
6. Write a dependency-ordered `plan.md`. Add `work-packages.md` only for independently implementable, non-overlapping path ownership.
7. Keep `proposal.md` at `Status: proposed` throughout authoring.

For an intent delta, author only whole Claim additions or supported criticality changes. A Claim
addition uses:

```markdown
# Intent delta: <spec-id>

## Add claim: <claim-id>
Criticality: routine

Non-empty free-form normative Markdown stating this Claim.

### Add case: <case-id>
Non-empty free-form normative Markdown stating this Case.
```

Claim and Case bodies may use any human language, EARS, unrestricted prose, tables, diagrams or
code fences. Core preserves and fingerprints their Markdown but does not interpret keywords,
translations or notation. Reserve the top three structural headings for Spec, Claim or Invariant,
and Case declarations; use level-four headings or fenced content inside a body. Keep orientation,
rationale and generated duplicate views outside normative bodies. Use lower-kebab declarative ids,
put universal meaning in the Claim, make each Case stand alone, and never author test mechanics as
intent. The installed `intent-delta` reference remains the complete release-matched operation set.

## Validate and hand off

Run `azimuth change check <id>`, `azimuth change work-packages <id>` when present, the repository's normal `azimuth validate`, and `azimuth change show <id>`. Inspect the working diff and ask whether an implementer can proceed without inventing product behavior, ownership, compatibility, migration, failure or completion decisions. Present the actual files for explicit approval. Do not implement, finalize, archive or commit implicitly.
