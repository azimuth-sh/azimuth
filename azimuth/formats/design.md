# Design

The mechanism facet (D3): **what makes a claim true, and how strongly.**

Design documents rot as a rule. Two things are meant to prevent it here, and they are the whole
justification for the format:

1. **Nothing structural is written.** "The trip service calls the payments client" is derivable
   from the code and the `realizes` tags. If a machine could produce a line, the line does not
   belong here.
2. **Every entry is a falsifiable assertion about a named artifact.** An entry claims that a
   specific index, type, or function is what holds a claim up. When the code stops matching, that
   is a **hole**, not stale prose. This is what design documents have never had, and it is why
   this one can be checked instead of believed.

Required for `critical` requirements, optional for `standard`, absent for `routine` (D6.5).

`design.md` describes **accepted current mechanisms** for its sibling spec. Proposed architecture,
alternatives and implementation sequencing live in `azimuth/changes/<id>/design.md` (D21.2), where
a planned mechanism may honestly be absent. Archiving distils only what was built into the model;
copying an abandoned plan there would turn change history into design fiction.

## File

```markdown
# Design: <spec-id>
```

At most one `design.md` sits beside a package's `spec.md` and declares the same spec id. Its
presence is governed by criticality; path is convention and declared id is identity.

A mechanism specific to one business rule lives with that rule's design. A reusable control such
as authentication, rate limiting or circuit breaking belongs in a concern-oriented spec/design
(`security/authentication`, `resilience/dependency-failures`), not copied into each consuming
feature. How such a mechanism proves its application across several specs is intentionally still
open; D27 requires the next real adoption to establish that relation before the framework adds a
catalog or composition syntax.

## Entries key on the requirement, not the scenario

Verification entries key on scenarios because the scenario is the unit of coverage. Mechanism
entries key on requirements because that is usually where the mechanism operates: one unique
index makes all three `captured-once` scenarios true, and recording it three times would be the
duplication this framework exists to prevent.

The claim tree has two levels, and **a facet attaches at the coarsest level where its statement is
true.** An entry may key on a scenario where the mechanism genuinely differs per scenario.

## Entries

```markdown
## Requirement: <requirement-id>
Mechanism: <stable-mechanism-id>
Enforcement: <kind>
Binding: <optional machine-addressable artifact id emitted by an extractor>
Expect: <optional derived properties that must match, such as uniqueness, columns or predicate>

Prose: why this mechanism, what was rejected and why, and what breaks if it changes. Required.
An entry that states a mechanism without a reason records a fact the code already knows.
```

A requirement may carry several named mechanisms, in order, where more than one holds it up. Each
mechanism id is stable and unique within its design. C2 in the concern catalog is the worked
example: a choke point *and* a representation constraint, for one rule.

### Enforcement kinds

The closed set, ordered by D7's ladder:

| Rung | Kind | Violation is | Derived strength |
|---|---|---|---|
| 1 | `type` | unrepresentable in the type system | proof |
| 1 | `schema` | unrepresentable in the data schema | proof |
| 2 | `constraint` | rejected by storage — unique index, FK, check, RLS | proof |
| 2 | `choke-point` | only possible through one place | proof |
| 3 | `middleware` | prevented where applied, and application is opt-in | demonstration required |
| 4 | `guard` | checked at each site | demonstration required |

**Strength is never written.** It is derived from the kind (D7): the top two rungs *are*
proof-strength evidence, which is why a claim enforced at rung 1 or 2 can carry a weaker evidence
requirement without that being a bargain. Writing it would duplicate a derivable fact.

### Identity and binding

`Mechanism:` is the independent conceptual anchor. Production code refers to it with
`ImplementsMechanism(spec, mechanism)`; the extractor derives the exact symbol binding. Removing
the marker or its code then leaves the design declaration unresolved. A rename with the marker
intact changes the derived binding and expires judgments that read that source instead of leaving
stale symbol prose.

`Binding:` remains available for non-code artifacts, such as an index emitted from migration
metadata. It is also accepted for code during incremental adoption. Exactly one binding must
resolve: either one explicit binding or one extractor-derived implementation. Zero is an
`unresolved-design-binding`; two or more are ambiguous, so a mechanism spanning two atomic sites
must be declared as two mechanisms.

`Binding:` is structural and exact; the paragraph below it carries the human explanation. A
symbol binding establishes that the symbol exists, not that words such as “only” or “every” are
true. A schema binding may additionally carry an `Expect:` line because uniqueness, columns and a
predicate are derivable from migration metadata and can therefore be compared exactly.

```markdown
Mechanism: unique-quote-consumption
Enforcement: constraint
Binding: postgres-index:trips.ux_trip_quote
Expect: unique=true; columns=quote_id
```

Precise enough for the check to be mechanical:

| Kind | `Binding` names | The check asks |
|---|---|---|
| `type` | the type claimed to make a value impossible | does the compiled type exist and is the binding a type rather than a method |
| `schema` | the schema element | was that exact schema artifact emitted |
| `constraint` | the constraint by name and table | does it exist in the migrations, with that uniqueness or predicate |
| `choke-point` | the function or module claimed to be singular | does the compiled operation exist and is the binding an operation rather than a type |
| `middleware` | the registration claimed to cover a surface | does the named artifact exist |
| `guard` | the operation claimed to check a rule | does the named artifact exist |

Only migration-derived index properties are checked semantically in the current machine tier.
Symbol existence prevents fiction and some category errors; it does not establish “only caller,”
absence of an escape hatch, complete middleware coverage or correct guard logic. Those assertions
remain part of the agent judgment until a purpose-built analyzer can derive them. Middleware and
guard coverage also need a sound enumerator (D13.1); a hand-listed set is worse than no check.

### Mechanism evidence

`CoversMechanism(spec, mechanism, scope, quantification, oracle)` marks evidence about the
mechanism's own contract—for example, a circuit breaker's state transitions. It is separate from
`Covers(spec, scenario, ...)`, which supports a business claim. A shared mechanism test is not
duplicated for every dependent scenario, but it also does not automatically cover those scenarios:
the relation proving that a mechanism is applied to every relevant domain member is not yet in the
model. The next real mechanism change is intended to validate that composition before it becomes
syntax.

## Residue

```markdown
## Residue
Free prose. Never parsed. Never derived.
```

Orientation, danger zones, deliberately broken corners, what is absent and why. This attaches to
no claim, participates in no check, and is the durable half — no reflection recovers "this lost
update is intentional, not a bug."

**Not the same as a verification residual.** A residual in sibling `verification.md` records
evidence that is missing. Residue here records judgment that cannot be derived. The first is a gap;
the second is knowledge.

Marking it explicitly is what stops the design file becoming a dumping ground: anything that is
neither a checkable mechanism claim nor deliberate judgment does not belong in either section.
