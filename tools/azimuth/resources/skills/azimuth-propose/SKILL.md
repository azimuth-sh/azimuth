---
name: azimuth-propose
description: Create or revise one bounded, approval-ready Azimuth change from a clear request or approved exploration. Use to establish singular authority, author current intent deltas, decide solution boundaries and validate a comprehensive proposal before implementation.
---

# Propose one change

A change proposal defines what should change in the product, why, and what would justify accepting it—including proposed intent, scope, evidence requirements, design decisions, and delivery plan. Use the term “change proposal” consistently.

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

For an intent delta, author only operations supported by `azimuth reference show intent-delta`.
Run `azimuth change intent-capture <id>` for replacements and removals so the reviewed delta pins
the accepted blocks it changes. A Claim addition uses:

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

Run `azimuth change intent-preview <id> --out <file>` and review the exact target spec and diff
with the proposal. Keep the preview as a reviewable artifact of the change. For edits to an
existing spec, inspect preserved surrounding Claims, explicitly removed Cases and affected
references. Preview states a target, not completed or accepted behavior.

## Author candidate Claim-first design

When the project uses the candidate Claim-first design format, a selected Mechanism may sit directly under its Claim design at level three. Prefer this placement before Case designs when the control contributes to several Cases. Declare its stable project-wide lower-kebab ID and optional explicit Cases metadata using stable Case IDs. Module membership and Case ownership do not determine identity. Nesting and order do not establish applicability or evidence. Elements still require a Section or Case design owner. Consult contracts/account-design.md for structural grammar.

Use ordinary Markdown captions for candidate verification prose groups; do not declare a Section identity just to organize Checks. Keep each Check one heading level below its structural owner or an intervening caption, through level six. Caption ancestry preserves Claim/Case ownership; a sibling caption closes prior scope at its level. Retain explicit Sections and Elements where named support is required. Captions and order supply no evidence.

Declare Claims, Cases, Mechanisms and Checks with stable project-wide lower-kebab IDs, unique within their entity kind. Preserve those IDs when splitting or merging modules or reparenting a Case. Module headers and Case nesting author mutable relationships only. Source APIs carry one stable target ID; contextual locators explicitly retain project, entity kind, ID and optional revision. Do not invent module-derived IDs, aliases, UUID requirements or cross-kind uniqueness.

For candidate verification Checks, Mechanism references and inline Case bindings refer directly to stable project-wide IDs. Validate target existence and duplicates without deriving identities from the document module or enclosing Claim. A dependency does not supply evidence or qualification.

## Review authored intent

Before validation, read each proposed Claim and its Cases as an independent reviewer. Apply these semantic criteria alongside the installed format reference:

- Give each Claim one independently governable, falsifiable proposition. Name its lower-kebab id as a compressed declaration of what is true, not as a topic noun or an implementation step. Split Claims when separate obligations could succeed, fail, or need different criticality independently.
- Put the general domain and outcome in the Claim. Keep implementation mechanisms, entity-field decisions, test procedures, and rationale in design or plan unless they are themselves the externally observable obligation.
- Give each Case a stand-alone declarative id and a concrete condition and outcome within its parent Claim. Include meaningful failure or boundary conditions. A Case that merely paraphrases the Claim, bundles several unrelated situations, or has no observable result needs revision.
- Prefer a Given/When/Then shape when it clarifies a Case: state the relevant starting condition, the request or event, and the observable result. Name a response status or other wire detail when it is a deliberate contract, and state the protected effect or absence of one on a denial. Split materially different conditions or outcomes into separate Cases instead of hiding them in one broad sentence. Keep test setup, probes and observation methods in verification.
- Check Claim-to-Case composition: Cases sample the Claim without narrowing its universal meaning to one example or pretending that one result proves the whole Claim. Given/When/Then is authoring guidance, not parser syntax or a required template; use equivalent prose when it is clearer.

Structural validation cannot perform this semantic review. Resolve weak or ambiguous wording before requesting approval, even if `azimuth change check` passes.

## Validate and hand off

Run `azimuth change check <id>`, `azimuth change work-packages <id>` when present, the repository's normal `azimuth validate`, and `azimuth change show <id>`. Inspect the working diff and ask whether an implementer can proceed without inventing product behavior, ownership, compatibility, migration, failure or completion decisions. Present the actual files for explicit approval. Do not implement, finalize, archive or commit implicitly.

## Author compact verification packages

Use compact Check method prose and inline Evidence bindings when the project adopts that account. Keep the Contribution narrower than the whole Case where the observation is narrower. Inherit Case only through its structural Case owner; use an explicit Case for shared Checks. Do not synthesize qualification, applicability or judgments from implementation tags.

Surface and network package declarations follow the installed verification reference. Enable packages explicitly, publish field names without duplicating language types, link a source producer through its package marker, and pin independently authored expectation data. State discovery completeness, controls and inconclusive conditions in module prose. Keep package mechanics in the package reference rather than repeating them in every module. Ordinary captions have no IDs.
