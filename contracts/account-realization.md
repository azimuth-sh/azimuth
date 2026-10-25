# Requirement-first realization format

This is the parser contract for a maintained `realization.md` beside a module's `spec.md` that uses Requirement-first headings. A `realization.md` containing at least one `## Requirement realization:` heading is parsed with this contract; any other `realization.md` uses the entry format in [realization.md](realization.md). The document explains how the implementation makes its Requirements hold. It may name selected controls and solution elements, but does not have to decompose the whole implementation into either. Tooling integrates the format into structural account assembly, model validation and Realization Review dependency fingerprints.

The file is optional. No review declaration requires a Requirement realization; a Requirement that declares `Realization: reviewed` without one is reviewed against its Requirement text and Scenarios alone. When a Requirement realization exists, the Realization Review checks the code at the realization sites against it, and its exact source is a blocking dependency of that review ([assurance-reviews.md](assurance-reviews.md)).

## Document and identity

```markdown
# Realization: auth/cli

Module orientation in ordinary Markdown.

## Section: shared-controls

Shared explanatory prose.

### Mechanism: consumed-refresh-recognition

Why this selected control matters, and what can fail.

## Requirement realization: refresh-rotates-credentials-and-revokes-reuse

- Areas:
  - `gateway`
  - `api`

The Requirement's explanation.

### Mechanism: refresh-admission

- Scenarios:
  - `replayed-refresh`

Why this selected control contributes to the explicitly listed Scenarios.

### Scenario realization: replayed-refresh

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

The declared module id must equal the owning module's spec id and supplies mutable membership, not identity. Requirement, Scenario and Mechanism declarations refer to stable project-wide lower-kebab IDs directly. Scenario nesting separately declares its current parent Requirement. Duplicate IDs within one entity kind anywhere in the assembled project fail; equal names across kinds remain distinct. Moving declarations to another module or changing a Scenario parent does not change entity identity. Sections and Elements retain their separate module-local vocabulary. See [entity-identity.md](entity-identity.md).

`Areas:` on every Requirement realization is a nonempty, duplicate-free set of workspace Area ids. These are the Requirement's production-realization participation obligation. Area definitions and mounts remain in `workspace.json`; no second authored obligation belongs there. An Area requires a production `Realizes` site but does not establish behavioral completeness.

`Element:` is optional. It requires one `Kind:` value, and may list `Areas:` and `Writes:` values. Kinds are descriptive ids, not a closed mechanism ladder. `Writes:` names Elements declared in the same realization. An Element may describe a logical store, operation, transaction, job or other useful part; one element need not map to one source symbol. No Element is required per Requirement or Area.

`Mechanism:` is optional and requires explanatory prose. Its stable ID names the selected control independently of module placement. Optional nonempty, duplicate-free `Scenarios` metadata refers directly to stable Scenario IDs anywhere in the project. These explicit relationships declare relevance, not evidence or establishment of a Scenario. Nesting supplies ownership context without inventing Scenario relationships. Omitting `Scenarios` leaves applicability unspecified. Account inspection rejects undeclared Scenario IDs. Source markers name only the stable Mechanism ID with `ImplementsMechanism(<mechanism-id>)`.

## Nesting and prose

A level-two `Requirement realization` or `Section` opens a top-level account or shared explanation. A Scenario realization is a level-three declaration directly under a Requirement realization. A `Section` may be nested one level below a Requirement realization, Scenario realization or another Section. `Element` declarations sit one level below a Section or Scenario realization, up to Markdown level six; they never sit directly under a Requirement realization. `Mechanism` declarations may also sit directly under a Requirement realization at level three; under a Section or Scenario realization they sit one level below that owner, up to Markdown level six. A control contributing to several Scenarios can be a Requirement-level sibling before their Scenario realizations rather than appearing owned by one Scenario. Its optional explicit `Scenarios` metadata determines applicability; its placement or order does not. Nesting gives a reading path; explicit relationships provide obligations. A shared Section may contain an Element or Mechanism used by several Requirements.

Only the listed declaration headings are parsed. Other Markdown, including ordinary headings, tables, Mermaid diagrams, links, lists, blank lines and fenced code blocks, is retained as prose. Declaration-looking headings within triple-backtick or triple-tilde fences are ignored. The parser retains the exact source and the exact source span of each declaration for projection and fingerprinting; it does not interpret prose as a relationship.

A metadata block immediately follows its declaration heading, separated only by blank lines. It uses visible Markdown list fields with backtick-delimited scalar values. List fields use indented `  - ` items. Unknown or duplicate fields fail. Required values must be present. Later prose does not add metadata. The parser recognizes `Areas` on Requirement realization; `Kind`, `Areas`, `Writes` on Element; `Scenarios` on Mechanism; and no metadata fields on Section or Scenario realization.

The parser reports malformed declaration headings, invalid ids, invalid nesting, duplicate identities, missing or invalid metadata, unknown `Writes` targets and missing Mechanism explanation with file and line diagnostics. Cross-facet references, including Scenario realization and Mechanism Scenario existence, workspace Area existence, model Requirement existence, prose adequacy, source bindings and review status belong to assembly/validation, not this parser; assembly reports them as `unknown-realization-requirement`, `unknown-realization-scenario` and `unknown-realization-area`. A Mechanism with metadata still requires nonempty explanatory prose after that metadata.

## Fingerprints and review

Normalized declarations, explicit relationships and the exact authored source of a Requirement realization, its Scenario realizations and every Mechanism bearing on the Requirement form that Requirement's realization digest. Editing any of them stales the Requirement's Realization Review and never its Verification Review. Neither a Scenario realization nor Mechanism applicability supplies verification evidence, and no conclusion is inferred from nesting or a source tag.
