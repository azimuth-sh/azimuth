# Spec format

This is the strict parser contract for repository intent. Anything not described here is a parse error; the parser fails rather than guessing.

## Shape

```markdown
# Spec: <spec-id>

Optional non-normative ownership prose.

## Term: <term-id>

Non-empty module vocabulary definition in free-form Markdown. Repeat for each term.

## Requirement: <requirement-id>
Title: <optional concise authored title>
Criticality: critical | standard | routine

Non-empty free-form normative Markdown stating the Requirement.

### Scenario: <scenario-id>
Non-empty free-form normative Markdown stating this Scenario.
```

The parser reserves `# Spec:`, `## Term:`, `## Requirement:` or `## Invariant:` and `### Scenario:` headings for the
semantic structure. A Requirement or Scenario body ends at the next unfenced heading at one of those levels.
A Term definition ends at the next unfenced level-one or level-two heading; lower headings remain
part of its Markdown definition. Headings inside a triple-backtick fence remain body content.
Outer blank lines are removed; all other Markdown, including line breaks, tables, blockquotes,
lists, diagrams and code fences, is preserved and participates in fingerprints.

Core validates that each authored body is non-empty but does not interpret its natural language,
diagram, table or formal notation. Authors may use any human language, EARS, another notation or
unrestricted prose; none is a parser requirement and no language declaration changes core
behavior. Human and agent review own clarity, falsifiability, internal consistency and the honesty
of the Requirement-to-Scenario composition.

## Term

A Term gives a stable local name to vocabulary shared by the module's Requirements. It does not create a
Requirement, Scenario, evidence target or independently judged obligation. Term ids are unique within a spec
and use lower kebab-case. Definitions are authored meaning, so core preserves their Markdown
without interpreting it. A missing or duplicate definition is a parse error.

A Term can affect the interpretation of any Requirement or Scenario in its spec. Core cannot reliably infer
which prose uses which Term, so the sorted set of all term ids and definitions participates in every
Requirement and Scenario semantic digest for that spec. Editing a definition therefore conservatively
invalidates dependent qualifications and judgments; reordering Term blocks alone does not.

## Requirement and Scenario

A Requirement states one independently governable product or operational proposition. It owns
criticality, production realization, total Requirement Judgment and Requirement Assurance State. A Scenario is a
normative condition/outcome clause within that Requirement's predicate. It is addressable for evidence,
Run selection, Check Results and Verification Challenge impact but is not independently governed.

Requirements and Scenarios have stable project-wide lower-kebab IDs in separate typed namespaces. Module membership and Scenario parent relationships are authored separately. Evidence Bindings target stable Scenario IDs because one result must not appear to exhaust the broader Requirement.

## Title

`Title:` is optional display metadata, declared once in the header of a Requirement or site-domain Invariant. It contains 1–120 characters on one line. Titles appear in catalogs, lists and navigation; the normative statement remains authoritative. Titles do not participate in Requirement or Scenario semantic fingerprints. An untitled Requirement is displayed by its stable ID, never by an automatically generated summary. Editing only a title does not invalidate the normative Requirement definition used by Requirement Reviews. Titles do participate in the complete model export and model revision, so each immutable catalog remains bound to its authored display metadata.

## Identity

- Spec ids are declared, never derived from paths.
- Hierarchical `/` segments are part of the id and support id selection.
- Package layout is convention; a path/id mismatch is a warning.
- Requirement ids are unique across the complete project.
- Scenario ids are unique across the complete project, independently of parent Requirement.
- Moving an entity between modules or a Scenario between Requirements preserves its ID; relationship changes are reviewed separately.
- Entity IDs use one lower-kebab segment; module IDs may use lower-kebab path segments.

## Criticality

Every Requirement declares criticality. Absence is an `unclassified` Finding, not a default. Scenarios inherit
their parent Requirement's level.

| Level | Current intent | Realization | Verification |
|---|---|---|---|
| `critical` | format retained for future use | required | explicit declarations and policy |
| `standard` | format retained for future use | required | explicit declarations and policy |
| `routine` | current framework level | not required | inapplicable |

All current framework Requirements are routine during the fast-moving alpha. They owe no Realizes
linkage, Check, Evidence Binding, Method Qualification, Applicability Decision or Requirement Judgment.
An ordinary test for a routine Requirement is outside the Azimuth evidence graph and needs no exemption.

The parser retains non-routine levels so a later accepted change can raise criticality. Such a change must add the verification declarations required by the verification format; it cannot restore an older implicit evidence model.

## What Scenarios do not carry

- No evidence form. Shared scope, quantification, oracle and common context belong to a Method
  Qualification; edge proposition and Scenario-specific context belong to an Evidence Binding.
- No source path. Production linkage is extracted from Realizes markers.
- No execution state. Runs and their Verification Targets belong to the deferred execution plane.
- No cross-cutting role labels. Add notation only after structurally different prose concerns establish the need.
- No required natural-language shape. Core does not recognize `GIVEN`, `WHEN`, `THEN`, `shall` or
  translated equivalents as semantic keywords.

## Site-domain invariants

An invariant may replace authored Scenarios with a declared surface:

```markdown
## Invariant: position-confined-to-live-phases
Criticality: routine
Over: trips/rider-view
```

`Over:` names a surface in `azimuth/workspace.json`, never an informal domain or path. Each surface
contribution binds an area mount to an enumerator. The parser supplies one implicit Scenario, with the
same local id as the Requirement, so evidence and Run addressing remain uniform. A missing surface,
failed contribution or undischarged member becomes a distinct Finding.

## Style

- An id is a compressed proposition: `terminal-states-are-final`, not `termination`.
- Ids are declarative, never imperative.
- Scenario ids must stand alone wherever traceability reports them.
- Requirement and Scenario ids should remain visibly distinct.
- Requirement and Scenario bodies are normative even when they use prose, tables, diagrams or a domain
  notation. Orientation and rationale belong outside their bodies.
- Prefer singular, falsifiable statements and observable behavior over test mechanics.
- Universal meaning belongs in the proposition, not in a test example.
- A table, diagram or code fence inside a Requirement or Scenario body is normative content. A generated view
  must remain outside the body unless it deliberately becomes the sole authored authority.

Specs are organized by domain area rather than service topology. If one file grows too broad, split it into specs with new declared ids instead of inventing a multi-file spec.
