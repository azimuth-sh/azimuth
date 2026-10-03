# Claim-first design format (candidate)

This is the parser contract for a maintained `design.md` beside a module's `spec.md`. The document explains how the implementation makes its Claims hold. It may name selected controls and solution elements, but does not have to decompose the whole implementation into either. The candidate tooling integrates this format into structural account assembly, model validation and review-dependency fingerprints; it does not establish an accepted Claim Judgment.

## Document and identity

```markdown
# Design: auth/cli

Module orientation in ordinary Markdown.

## Section: shared-controls

Shared explanatory prose.

### Mechanism: consumed-refresh-recognition

Why this selected control matters, and what can fail.

## Claim design: refresh-rotates-credentials-and-revokes-reuse

- Areas:
  - `gateway`
  - `api`

The Claim's explanation.

### Mechanism: refresh-admission

- Cases:
  - `replayed-refresh`

Why this selected control contributes to the explicitly listed Cases.

### Case design: replayed-refresh

How the parts cooperate.

#### Element: refresh-transaction

- Kind: `transaction`
- Areas:
  - `api`
- Writes:
  - `grant-store`
  - `credential-store`

The logical transaction boundary and its failure behavior.

#### Mechanism: atomic-refresh-consumption

- Cases:
  - `replayed-refresh`

Why this selected control prevents a particular failure.
```

The declared module id must equal the owning module’s spec id and supplies mutable membership, not identity. Claim, Case and Mechanism declarations refer to stable project-wide lower-kebab IDs directly. Case nesting separately declares its current parent Claim. Duplicate IDs within one entity kind anywhere in the assembled project fail; equal names across kinds remain distinct. Moving declarations to another module or changing a Case parent does not change entity identity. Sections and Elements retain their separate module-local vocabulary. See `contracts/entity-identity.md`.

`Areas:` on every Claim design is a nonempty, duplicate-free set of workspace Area ids. These are the Claim's production-realization participation obligation. Area definitions and mounts remain in `workspace.json`; no second authored obligation belongs there. An Area requires a production `Realizes` site but does not establish behavioral completeness.

`Element:` is optional. It requires one `Kind:` value, and may list `Areas:` and `Writes:` values. Kinds are descriptive ids, not a closed mechanism ladder. `Writes:` names Elements declared in the same design. An Element may describe a logical store, operation, transaction, job or other useful part; one element need not map to one source symbol. No Element is required per Claim or Area.

`Mechanism:` is optional and requires explanatory prose. Its stable ID names the selected control independently of module placement. Optional nonempty, duplicate-free `Cases` metadata refers directly to stable Case IDs anywhere in the project. These explicit relationships declare relevance, not evidence or establishment of a Case. Nesting supplies ownership context without inventing Case relationships. Omitting `Cases` leaves applicability unspecified. Account inspection rejects undeclared Case IDs. Source markers name only the stable Mechanism ID.

## Nesting and prose

A level-two `Claim design` or `Section` opens a top-level account or shared explanation. A Case design is a level-three declaration directly under a Claim. A `Section` may be nested one level below a Claim, Case design or another Section. `Element` declarations sit one level below a Section or Case design, up to Markdown level six. `Mechanism` declarations may also sit directly under a Claim design at level three; under a Section or Case design they sit one level below that owner, up to Markdown level six. A control contributing to several Cases can be a Claim-level sibling before their Case designs rather than appearing owned by one Case. Its optional explicit `Cases` metadata determines applicability; its placement or order does not. Nesting gives a reading path; explicit relationships provide obligations. A shared Section may contain an Element or Mechanism used by several Claims.

Only the listed declaration headings are parsed. Other Markdown, including ordinary headings, tables, Mermaid diagrams, links, lists, blank lines and fenced code blocks, is retained as prose. Declaration-looking headings within triple-backtick or triple-tilde fences are ignored. The parser retains the exact source and the exact source span of each declaration for projection and future fingerprinting; it does not interpret prose as a relationship.

A metadata block immediately follows its declaration heading, separated only by blank lines. It uses visible Markdown list fields with backtick-delimited scalar values. List fields use indented `  - ` items. Unknown or duplicate fields fail. Required values must be present. Later prose does not add metadata. The current parser recognizes `Areas` on Claim design; `Kind`, `Areas`, `Writes` on Element; `Cases` on Mechanism; and no metadata fields on Section or Case design.

The parser reports malformed declaration headings, invalid ids, invalid nesting, duplicate identities, missing or invalid metadata, unknown `Writes` targets and missing Mechanism explanation with file and line diagnostics. Cross-facet references, including Case design and Mechanism Case existence, workspace Area existence, model Claim existence, prose adequacy, source bindings and review status belong to assembly/validation, not this parser. A Mechanism with metadata still requires nonempty explanatory prose after that metadata.

## Open integration decisions

Accepted Claim-first Judgment and Challenge integration must still decide how selected control participation composes into a total Claim conclusion. Explicit Case applicability, source implementations and substantive design prose are retained for inspection and review dependencies. No inference from nesting or a source tag settles these questions. The accepted design parser and its current model remain unchanged until that integration is reviewed.

## Decision revision: 2026-09-30

The initial candidate deliberately omitted Mechanism Case lists. The revised decision permits explicit optional applicability relationships because a selected control can contribute to only part of a Claim or to Cases under several Claims. Section-to-Case links remain explanatory prose. This revision adds reference validation without making mechanism participation count as Case evidence. Existing design-source fingerprints retain the exact authored scope and therefore change when it changes; accepted Claim-first Judgment and Challenge integration remains open.

## Contextual identity revision: 2026-09-30

Claim design, Claim verification and Claim review headings now use local Claim names within their module documents. Explicit Case design declarations replace explanatory Sections when the author intends a structural Case reference. Mechanism Case lists may use local names under a Claim and qualified names for cross-Claim or shared scope. Parsed identities and relationships are fully qualified before validation and fingerprinting; exact authored prose remains fingerprinted as well. No Case design is mandatory, and neither a Case design nor Mechanism applicability supplies verification evidence. This supersedes the earlier Section-only Case explanation and qualified-only scope decisions; legacy accepted design grammar is unchanged.

## Claim-level Mechanism revision: 2026-10-02

Dima approved Mechanisms directly under Claim design at Markdown level three. A control that contributes to several Cases can precede those Case designs as their sibling. Local Case references resolve against the same enclosing Claim; explicit applicability, stable Mechanism identity, source linkage and fingerprint semantics remain unchanged. This does not permit Elements directly under Claims or infer relationships from order or nesting. Legacy accepted design grammar remains unchanged.

## Contextual Mechanism declaration revision: 2026-10-02

Dima clarified that the Mechanism declaration itself should use a local name as well as local references. Candidate design accepts local declarations normalized against its declared module and still accepts explicitly qualified declarations for that same module. Mixed-spelling duplicates and qualified declarations belonging to another module fail at their source locations. Projection retains authored headings and guarded Parent locators; semantic identity and relationship fingerprints use normalized declarations. Exact authored-source fingerprints retain structural and spelling changes. Legacy accepted grammar is unchanged.

## Stable identity decision superseding qualified normalization: 2026-10-02

Dima approved the coordinated stable entity identity implementation under module-qualified-check-identities (its administrative name records the superseded initial decision). Claim, Case, Mechanism and Check IDs now remain independent of module membership and Case parent relationships. Earlier dated local/qualified normalization decisions and product observations above are historical; they do not describe current accepted parser syntax. Current declarations and references use stable project-wide lower-kebab IDs, source tags identify one stable target, and complete-project validation rejects same-kind duplicate IDs. No aliases or syntax-only alpha protocol version increases are introduced. Ordinary captions and Claim-level Mechanism structure remain valid.
