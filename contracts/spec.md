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
Realization: reviewed
Verification: reviewed
Context: <component-id>: <leaf> = <value>

Non-empty free-form normative Markdown stating the Requirement.

### Scenario: <scenario-id>
Context: <component-id>: <leaf> = <value>

Non-empty free-form normative Markdown stating this Scenario.
```

`Title:`, `Realization:` and `Verification:` are optional and appear at most once. `Context:` lines are optional and repeatable.

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
invalidates dependent reviews; reordering Term blocks alone does not.

## Requirement and Scenario

A Requirement states one independently governable product or operational proposition. It owns
its review declarations, production realization, its Realization and Verification Reviews and Requirement Status. A Scenario is a
normative condition/outcome clause within that Requirement's predicate. It is addressable for evidence,
Run selection, Check Results and Verification Challenge impact but is not independently governed.

Requirements and Scenarios have stable project-wide lower-kebab IDs in separate typed namespaces. Module membership and Scenario parent relationships are authored separately. Evidence Bindings target stable Scenario IDs because one result must not appear to exhaust the broader Requirement.

## Title

`Title:` is optional display metadata, declared once in the header of a Requirement or site-domain Invariant. It contains 1–120 characters on one line. Titles appear in catalogs, lists and navigation; the normative statement remains authoritative. Titles do not participate in Requirement or Scenario semantic fingerprints. An untitled Requirement is displayed by its stable ID, never by an automatically generated summary. Editing only a title does not invalidate the normative Requirement definition used by Realization and Verification Reviews. Titles do participate in the complete model export and model revision, so each immutable catalog remains bound to its authored display metadata.

## Context conditions

A `Context:` line states a condition on the effective Execution Context of a deployed component under which the Requirement or Scenario applies. The line is `Context: <component-id>: <leaf> = <value>`: the component id and leaf split at the first `: `, the leaf is a dotted name declared by that component's execution-context declaration ([deployment-requirement-status.md](deployment-requirement-status.md)), and `<value>` is `true`, `false`, an integer, or a double-quoted string with JSON escapes. A feature flag is such a leaf; Azimuth never sets one.

- Context lines occupy the header block of a Requirement (after `Title:`, `Realization:` and `Verification:`) and, new, the header block of a Scenario: the maximal run of consecutive `Context:` lines immediately after the heading, before the first blank line. A later line that happens to begin with `Context:` is body text. A Scenario whose body would begin with such a line must separate it from the heading with a blank line to remain body.
- All lines hold together (AND). There is no OR; author two Scenarios.
- Requirement-level lines apply to every Scenario of the Requirement. Scenario lines add to them. A Scenario cannot remove or override an inherited line: a repeated component and leaf with a different value, at either level, is `condition-conflict`. An identical repeat is a duplicate-line error.
- The lines are part of the Requirement and Scenario text and therefore of their semantic digests, so editing one makes the Requirement's Realization and Verification Reviews stale.
- Validation: the component exists in the project catalog's `components` ([project-catalog.md](project-catalog.md)); the leaf exists in that component's declaration; the value has the declared leaf type. Findings are `condition-unknown-component`, `condition-unknown-leaf`, `condition-type-mismatch` and `condition-conflict`, all errors. Deleting a leaf or a component from a declaration while any condition names it fails validation, which forces the condition lines out in the same change. A project whose catalog declares no `components` rejects every condition as `condition-unknown-component`.
- Intent deltas carry complete Requirement blocks, so `Context:` lines travel inside them; a delta parser that cannot represent them exactly is a defect, not a reason to drop the lines.

Conditions do not alter identity, review declarations or realization. Their meaning for Requirement Status is in [deployment-requirement-status.md](deployment-requirement-status.md): a deployment whose captured context does not satisfy a condition is shown as not applicable, never as a violation.

## Identity

- Spec ids are declared, never derived from paths.
- Hierarchical `/` segments are part of the id and support id selection.
- Package layout is convention; a path/id mismatch is a warning.
- Requirement ids are unique across the complete project.
- Scenario ids are unique across the complete project, independently of parent Requirement.
- Moving an entity between modules or a Scenario between Requirements preserves its ID; relationship changes are reviewed separately.
- Entity IDs use one lower-kebab segment; module IDs may use lower-kebab path segments.

## Review declarations

A Requirement declares, per review axis, whether it owes an independent review. The two declarations are
labelled lines in the Requirement header block, directly under the `## Requirement:` or `## Invariant:` heading,
before the first blank line and in any order relative to `Title:`:

```markdown
## Requirement: refresh-revokes-reuse
Title: Refresh revokes a reused credential
Realization: reviewed
Verification: reviewed
```

- `reviewed` is the only accepted value of either label. Any other value, including an empty one, is a parse
  error naming the label and the value.
- A label declared twice in one header is a parse error.
- An absent label means that axis is not declared; it is not a default and not a Finding.
- Scenarios inherit both declarations from their parent Requirement and carry neither label.
- Both declarations are part of the Requirement text and therefore of its semantic digest, so adding or removing
  one stales every review that depends on that text.

What each declaration owes:

| Declaration | Owes |
|---|---|
| `Realization: reviewed` | At least one production `Realizes(<requirement-id>)` site, `ImplementsMechanism` sites for every Mechanism its [realization.md](realization.md) declares when that file has entries for the Requirement, and a current accepted independent Realization Review ([assurance-reviews.md](assurance-reviews.md)). |
| `Verification: reviewed` | Enrolled Checks in the module's `verification.md` with an Evidence Binding for every Scenario ([account-verification.md](account-verification.md)), and a current accepted independent Verification Review. |

A Requirement that declares neither is specified: it states intent only and owes no realization linkage, Check,
Evidence Binding or review. Checks are not allowed for a Requirement without `Verification:`: a Check that targets
one of its Scenarios is a structural error ([account-verification.md](account-verification.md)). An ordinary test of such a Requirement is outside the
Azimuth evidence graph and needs no exemption. A `realization.md` entry is optional for every Requirement,
whatever it declares.

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
