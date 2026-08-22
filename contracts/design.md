# Design format

The design facet states which current mechanisms make Claims true. It records falsifiable,
non-derivable assertions about named artifacts rather than a structural tour of the code.

`design.md` describes accepted current mechanisms for its sibling spec. Proposed architecture,
alternatives and implementation order live in a change's `design.md`. Archiving distils only what
was built into the current model. A design may fan out verification obligations; it never fans out
evidence. Evidence exists once a Check, constraint or observation supplies it, so a planned
mechanism must never read as an achieved one.

All current framework Claims are routine and normally have no design obligation. The format
remains available for a later accepted non-routine Claim and for current mechanisms that genuinely
need durable explanation.

## File

```markdown
# Design: <spec-id>
```

At most one `design.md` sits beside a package's `spec.md` and declares the same spec id. Declared id
is authority; path is convention.

## Entries

```markdown
## Requirement: <requirement-id>
Mechanism: <stable-mechanism-id>
Enforcement: <kind>
Binding: <optional machine-addressable artifact id>
Expect: <optional derived properties>

Required prose explaining why this mechanism matters and what fails if it changes.
```

An entry attaches at the coarsest Claim level where its proposition is true. Mechanisms normally
key on a requirement because one constraint may support several cases. A mechanism that genuinely
varies per case may key on that case as allowed by the parser.

A requirement may have several ordered mechanisms. Each mechanism id is stable and unique within
its design. Reusable controls belong in concern-oriented specs instead of being copied into every
consumer package.

## Enforcement kinds

| Rung | Kind | Violation is |
|---|---|---|
| 1 | `type` | unrepresentable in the type system |
| 1 | `schema` | unrepresentable in the data schema |
| 2 | `constraint` | rejected by storage |
| 2 | `choke-point` | possible only through one place |
| 3 | `middleware` | prevented wherever the middleware is applied |
| 4 | `guard` | checked independently at each site |

Strength is derived from the kind and is never authored. Type, schema and constraint artifacts may
support structural proof in total Claim composition. They are mechanisms, not executable Checks;
inventing an execution result for them would confuse proof with sampled behavior.

## Identity and binding

`Mechanism:` is the conceptual anchor. Production code refers to it with
`ImplementsMechanism(spec, mechanism)`, and extractors derive a semantic source binding. `Binding:`
remains available for non-code artifacts such as an emitted database index.

Exactly one structural binding must resolve. Zero produces `unresolved-design-binding`; several
bindings are ambiguous, so a mechanism spanning independent atomic sites must be split.

`Expect:` compares properties an extractor can derive exactly. Current database-index support may
compare uniqueness, ordered columns and predicates:

```markdown
Mechanism: unique-quote-consumption
Enforcement: constraint
Binding: postgres-index:trips.ux_trip_quote
Expect: unique=true; columns=quote_id
```

A symbol binding establishes existence and category, not semantic claims such as “only caller”
or “every route.” A purpose-built analyzer or later reviewed composition format must establish
those stronger propositions.

Mechanism implementation linkage is structurally separate from Check implementation linkage.
Tests of mechanism behavior are ordinary engineering tests unless a future non-routine Claim has a
deliberate Check and Evidence Binding. No source marker assigns evidentiary meaning to a mechanism.

## Residue

```markdown
## Residue
Free prose. Never parsed or derived.
```

Residue holds durable orientation, danger zones and deliberately accepted absences. It attaches to
no Claim and participates in no fingerprint. Anything that is either derivable structure or an
unaccepted proposal does not belong here.
