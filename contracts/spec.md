# Spec format

This is the strict parser contract for repository intent. Anything not described here is a parse error; the parser fails rather than guessing.

## Shape

```markdown
# Spec: <spec-id>

Optional non-normative ownership prose.

## Requirement: <requirement-id>
Criticality: critical | standard | routine

A singular SHALL statement.

### Scenario: <case-id>
GIVEN <precondition>          (optional, repeatable with AND)
WHEN <trigger>
THEN <observable outcome>
AND <further outcome>         (optional, repeatable)
```

## Two Claim levels

A requirement-level Claim states the normative product or operational proposition and owns criticality. A scenario declares a case-level Claim that refines one observable condition. Both are Claims; scenario syntax does not create a separate ontology.

Requirement identity is `<spec>#<requirement>`. Case identity is `<spec>#<case>`. Evidence Bindings target case-level Claims because one result must not appear to exhaust a broader requirement.

## Identity

- Spec ids are declared, never derived from paths.
- Hierarchical `/` segments are part of the id and support id selection.
- Package layout is convention; a path/id mismatch is a warning.
- Requirement ids are unique within a spec.
- Case ids are unique within a spec, not merely within a requirement.
- Moving a case between requirements preserves its case identity.
- Ids use lower kebab path segments and name falsifiable propositions.

## Criticality

Every requirement declares criticality. Absence is an `unclassified` Finding, not a default. Cases inherit their parent requirement's level.

| Level | Current intent | Realization | Verification |
|---|---|---|---|
| `critical` | format retained for future use | required | explicit declarations and policy |
| `standard` | format retained for future use | required | explicit declarations and policy |
| `routine` | current framework level | not required | inapplicable |

All current framework Claims are routine during the fast-moving alpha. They owe no Realizes linkage, Check, Evidence Binding or Qualification. An ordinary test for a routine Claim is outside the Azimuth evidence graph and needs no exemption.

The parser retains non-routine levels so a later accepted change can raise criticality. Such a change must add the verification declarations required by the verification format; it cannot restore an older implicit evidence model.

## What cases do not carry

- No evidence form. Scope, quantification, oracle and required context belong to an Evidence Binding, not to the Claim.
- No source path. Production linkage is extracted from Realizes markers.
- No execution state. Runs and their Subjects belong to the deferred execution plane.
- No cross-cutting role labels. Add notation only after structurally different prose concerns establish the need.

## Site-domain invariants

An invariant may replace scenarios with a declared surface:

```markdown
## Invariant: position-confined-to-live-phases
Criticality: routine
Over: trips/rider-view
```

`Over:` names a surface in `azimuth/workspace.json`, never an informal domain or path. Each surface contribution binds an area mount to an enumerator. A missing surface, failed contribution or undischarged member becomes a distinct Finding.

## Style

- An id is a compressed proposition: `terminal-states-are-final`, not `termination`.
- Ids are declarative, never imperative.
- Case ids must stand alone wherever traceability reports them.
- Requirement and case ids should remain visibly distinct.
- SHALL statements are singular and normative.
- Cases describe observable behavior, never test mechanics.
- Universal meaning belongs in the proposition, not in a test example.
- Diagrams are non-normative and ignored by the parser.

Specs are organized by domain area rather than service topology. If one file grows too broad, split it into specs with new declared ids instead of inventing a multi-file spec.
