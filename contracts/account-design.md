# Requirement-first design format (candidate)

This is the parser contract for a maintained `design.md` beside a module's `spec.md`. The document explains how the implementation makes its Requirements hold. It may name selected controls and solution elements, but does not have to decompose the whole implementation into either. The candidate tooling integrates this format into structural account assembly, model validation and review-dependency fingerprints; it does not establish an accepted Requirement Judgment.

## Document and identity

```markdown
# Design: auth/cli

Module orientation in ordinary Markdown.

## Section: shared-controls

Shared explanatory prose.

### Mechanism: consumed-refresh-recognition

Why this selected control matters, and what can fail.

## Requirement design: refresh-rotates-credentials-and-revokes-reuse

- Areas:
  - `gateway`
  - `api`

The Requirement's explanation.

### Mechanism: refresh-admission

- Scenarios:
  - `replayed-refresh`

Why this selected control contributes to the explicitly listed Scenarios.

### Scenario design: replayed-refresh

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

- Scenarios:
  - `replayed-refresh`

Why this selected control prevents a particular failure.
```

The declared module id must equal the owning module’s spec id and supplies mutable membership, not identity. Requirement, Scenario and Mechanism declarations refer to stable project-wide lower-kebab IDs directly. Scenario nesting separately declares its current parent Requirement. Duplicate IDs within one entity kind anywhere in the assembled project fail; equal names across kinds remain distinct. Moving declarations to another module or changing a Scenario parent does not change entity identity. Sections and Elements retain their separate module-local vocabulary. See `contracts/entity-identity.md`.

`Areas:` on every Requirement design is a nonempty, duplicate-free set of workspace Area ids. These are the Requirement's production-realization participation obligation. Area definitions and mounts remain in `workspace.json`; no second authored obligation belongs there. An Area requires a production `Realizes` site but does not establish behavioral completeness.

`Element:` is optional. It requires one `Kind:` value, and may list `Areas:` and `Writes:` values. Kinds are descriptive ids, not a closed mechanism ladder. `Writes:` names Elements declared in the same design. An Element may describe a logical store, operation, transaction, job or other useful part; one element need not map to one source symbol. No Element is required per Requirement or Area.

`Mechanism:` is optional and requires explanatory prose. Its stable ID names the selected control independently of module placement. Optional nonempty, duplicate-free `Scenarios` metadata refers directly to stable Scenario IDs anywhere in the project. These explicit relationships declare relevance, not evidence or establishment of a Scenario. Nesting supplies ownership context without inventing Scenario relationships. Omitting `Scenarios` leaves applicability unspecified. Account inspection rejects undeclared Scenario IDs. Source markers name only the stable Mechanism ID.

## Nesting and prose

A level-two `Requirement design` or `Section` opens a top-level account or shared explanation. A Scenario design is a level-three declaration directly under a Requirement. A `Section` may be nested one level below a Requirement, Scenario design or another Section. `Element` declarations sit one level below a Section or Scenario design, up to Markdown level six. `Mechanism` declarations may also sit directly under a Requirement design at level three; under a Section or Scenario design they sit one level below that owner, up to Markdown level six. A control contributing to several Scenarios can be a Requirement-level sibling before their Scenario designs rather than appearing owned by one Scenario. Its optional explicit `Scenarios` metadata determines applicability; its placement or order does not. Nesting gives a reading path; explicit relationships provide obligations. A shared Section may contain an Element or Mechanism used by several Requirements.

Only the listed declaration headings are parsed. Other Markdown, including ordinary headings, tables, Mermaid diagrams, links, lists, blank lines and fenced code blocks, is retained as prose. Declaration-looking headings within triple-backtick or triple-tilde fences are ignored. The parser retains the exact source and the exact source span of each declaration for projection and future fingerprinting; it does not interpret prose as a relationship.

A metadata block immediately follows its declaration heading, separated only by blank lines. It uses visible Markdown list fields with backtick-delimited scalar values. List fields use indented `  - ` items. Unknown or duplicate fields fail. Required values must be present. Later prose does not add metadata. The current parser recognizes `Areas` on Requirement design; `Kind`, `Areas`, `Writes` on Element; `Scenarios` on Mechanism; and no metadata fields on Section or Scenario design.

The parser reports malformed declaration headings, invalid ids, invalid nesting, duplicate identities, missing or invalid metadata, unknown `Writes` targets and missing Mechanism explanation with file and line diagnostics. Cross-facet references, including Scenario design and Mechanism Scenario existence, workspace Area existence, model Requirement existence, prose adequacy, source bindings and review status belong to assembly/validation, not this parser. A Mechanism with metadata still requires nonempty explanatory prose after that metadata.

## Open integration decisions

Accepted Requirement-first Judgment and Challenge integration must still decide how selected control participation composes into a total Requirement conclusion. Explicit Scenario applicability, source implementations and substantive design prose are retained for inspection and review dependencies. No inference from nesting or a source tag settles these questions. The accepted design parser and its current model remain unchanged until that integration is reviewed.

## Decision revision: 2026-09-30

The initial candidate deliberately omitted Mechanism Scenario lists. The revised decision permits explicit optional applicability relationships because a selected control can contribute to only part of a Requirement or to Scenarios under several Requirements. Section-to-Scenario links remain explanatory prose. This revision adds reference validation without making mechanism participation count as Scenario evidence. Existing design-source fingerprints retain the exact authored scope and therefore change when it changes; accepted Requirement-first Judgment and Challenge integration remains open.

## Contextual identity revision: 2026-09-30

Requirement design, Requirement verification and Requirement review headings now use local Requirement names within their module documents. Explicit Scenario design declarations replace explanatory Sections when the author intends a structural Scenario reference. Mechanism Scenario lists may use local names under a Requirement and qualified names for cross-Requirement or shared scope. Parsed identities and relationships are fully qualified before validation and fingerprinting; exact authored prose remains fingerprinted as well. No Scenario design is mandatory, and neither a Scenario design nor Mechanism applicability supplies verification evidence. This supersedes the earlier Section-only Scenario explanation and qualified-only scope decisions; legacy accepted design grammar is unchanged.

## Requirement-level Mechanism revision: 2026-10-02

Dima approved Mechanisms directly under Requirement design at Markdown level three. A control that contributes to several Scenarios can precede those Scenario designs as their sibling. Local Scenario references resolve against the same enclosing Requirement; explicit applicability, stable Mechanism identity, source linkage and fingerprint semantics remain unchanged. This does not permit Elements directly under Requirements or infer relationships from order or nesting. Legacy accepted design grammar remains unchanged.

## Contextual Mechanism declaration revision: 2026-10-02

Dima clarified that the Mechanism declaration itself should use a local name as well as local references. Candidate design accepts local declarations normalized against its declared module and still accepts explicitly qualified declarations for that same module. Mixed-spelling duplicates and qualified declarations belonging to another module fail at their source locations. Projection retains authored headings and guarded Parent locators; semantic identity and relationship fingerprints use normalized declarations. Exact authored-source fingerprints retain structural and spelling changes. Legacy accepted grammar is unchanged.

## Stable identity decision superseding qualified normalization: 2026-10-02

Dima approved the coordinated stable entity identity implementation under module-qualified-check-identities (its administrative name records the superseded initial decision). Requirement, Scenario, Mechanism and Check IDs now remain independent of module membership and Scenario parent relationships. Earlier dated local/qualified normalization decisions and product observations above are historical; they do not describe current accepted parser syntax. Current declarations and references use stable project-wide lower-kebab IDs, source tags identify one stable target, and complete-project validation rejects same-kind duplicate IDs. No aliases or syntax-only alpha protocol version increases are introduced. Ordinary captions and Requirement-level Mechanism structure remain valid.
