# Verification plans

The evidence facet (D3). A plan records **what would be sufficient to believe a claim** — never
what currently exists. The evidence that exists is derived from `covers` tags or imported external
evidence receipts and appears in the export; hand-listing it would create a second copy that drifts
(D4.5).

Only standard and critical claims owe evidence. Routine claims stop at intent and need no
`covers` tags (D20). A test without `covers` is an ordinary test outside Azimuth's evidence model,
not an untraced evidence item and not an exemption (D20.1).

## What a plan contains

Only what is not derivable:

1. **Deviations from the project standard** — a claim needing stronger or different evidence than
   its criticality implies.
2. **Non-test evidence** — proof by construction, monitors, manual passes, attestations. Nothing
   in the code says "this unique index is the evidence for that claim".
3. **Residual risk** — what is knowingly not covered, and why that is acceptable.

**A claim with no entry is not unplanned.** It means the project standard applies unmodified.
Most claims should have no entry; if most have one, either the standard is wrong or the plan is
being used as an inventory.

Imported load, chaos, recovery and similar executions use D39's assurance-observation protocol.
Each evidence binding declares its actual form and becomes ordinary `covers`; one execution may
bind to several claims, but no run-level pass supplies blanket coverage. Mutation and broad static
analysis normally bind as judgment challenges instead and therefore do not appear as evidence.

## Evidence ownership and authorship

The evidence owner is accountable for what would justify belief and how it stays fresh (D30). That
does not require one QA role to write every test. A developer may implement unit or component
evidence, a quality engineer may implement it directly or direct an agent, and an SRE may implement
and operate an alert. Test code receives ordinary engineering review; the evidence owner remains
accountable for whether the result rejects a plausible wrong implementation and honestly declares
its scope, quantification and oracle.

The converse handoff is also invalid: mechanism authors cannot treat a downstream evidence owner
as solely responsible for quality. They provide testability, telemetry, controllable failure seams
and implementation expertise. `docs/change-process.md` gives the normal division by evidence form.

## File

```markdown
# Verification: <spec-id>
```

At most one `verification.md` sits beside a package's `spec.md` and declares the same spec id. A
spec needing no deviations, carrying no non-test evidence and accepting no residual needs **no
plan file at all**.

## Entries

```markdown
## Claim: <scenario-id>
Scope: unit | component | e2e
Quantification: example | universal
Oracle: direct | golden | relational | metamorphic | model-based | contract

Prose stating why this claim needs what it needs. Required — an entry without a reason is a
number nobody can review.
```

Labels first, then a blank line, then prose. Values may wrap: inside the label block, a line that
begins no known label continues the previous one. Every field is optional; an entry states only
what it changes. `Oracle` is descriptive and never gated.

Oracle names are distinguished by the source that can make the assertion fail:

| Oracle | Meaning | Typical shape |
|---|---|---|
| `direct` | The expected value is written in the evidence | `status == 404` |
| `golden` | The expected value is a recorded prior output | snapshot comparison |
| `relational` | Values observed for one case must satisfy a stated relation | total = sum(parts) |
| `metamorphic` | A relation holds across transformed executions | mutate token → rejection |
| `model-based` | An independent model computes the exact result | transition table vs subject |
| `contract` | An agreed interface or protocol supplies the expectation | schema or wire contract |

One test may contain more than one shape; its tag names the oracle doing the discriminating work
for this claim. The parser rejects unknown names, while adequacy remains an agent judgment. Oracle
kinds are categories, not a ladder, and the machine does not compare one as stronger than another.

**Two field groups that are easy to confuse, and are therefore kept apart:**

- `Scope`, `Quantification` and `Oracle` state the **required** form, overriding the standard.
- `Evidence` and its `Strength` declare a **provided** non-test evidence item.

`Strength` without `Evidence` is an error, because on its own it reads as either.

### Ladders

`Scope` and `Quantification` are ladders: `unit < component < e2e`, `example < universal`.
Strength is a ladder too: `detection < demonstration < proof`. **A stronger form on any axis
satisfies a requirement for a weaker one.** A required form is a floor, not a target.

Scope is defined by what must be *real* (D15) and applies to demonstration-strength evidence
only. Proof has no scope; detection has a target.

### Non-test evidence

```markdown
## Claim: <scenario-id>
Strength: proof
Evidence: partial unique index `ux_capture_trip` on `captures(trip_id)`
Binding: postgres-index:captures.ux_capture_trip

Violation is unrepresentable at the storage layer, so no execution can exhibit it.
```

For detection-strength items, two further fields are required (D4.3):

```markdown
Re-established: continuously | every release | quarterly
Dies silently: <how this stops being evidence without anyone noticing>
Detector test: <the test proving it fires on an injected violation>
Binding: <artifact id for the detector>
Detector binding: <artifact id for the detector test>
```

A detection item without its freshness account, detector test, detector binding or evidence
binding fails parsing. A named binding that no extractor emits is a hole. This establishes that the
rule and test exist; whether they detect the right condition remains an agent judgment. A monitor
that can no longer fire is worse than no monitor, because it is carried on the books as evidence.

The current machine tier checks repository-owned detector definitions, bindings and synthetic
detector behaviour. It does not query a live metrics backend to establish scrape health, rule
evaluation or notification routing. `Re-established:` and `Dies silently:` therefore expose an
operating obligation rather than pretending the repository proves it. A provider adapter may
later import expiring operational receipts through the same language-neutral seam as manual
results.

### Judging manual and operational evidence

The agent judgment is per claim, not a separate verdict per test, receipt or monitor. Those items
are fingerprinted inputs to the claim's complete assurance account. A new receipt, detector, plan
or bound source makes an older judgment stale.

For a manual receipt, the judge needs the linked procedure and observations, not only the external
system's `passed` status. The procedure must reject a plausible wrong behaviour and its declared
scope and quantification must match what the person executed. If the linked record is inaccessible,
the machine can establish attribution and freshness but the agent cannot responsibly call the
evidence toothy.

For operational evidence, the judge reads the metric producer, rule expression, threshold,
detector test and stated silent-death modes. It asks whether the injected failure is the claimed
violation and whether a broken producer could leave the detector quietly green. Detection remains
evidence about learning of a violation; it never becomes proof that the production property holds.

### Lowering a requirement

A plan may require *less* than the project standard, but only with an accepted residual:

```markdown
## Claim: <scenario-id>
Quantification: example
Residual: not checked across all currencies
Accepted: single-currency market until the second market launches; revisit then
```

Silent weakening is not available. This is D6.3's exemption principle applied to evidence: a
deliberate, attributable, reviewable opt-out is fine anywhere; an unrecorded absence is not.

### Spec-level residual

```markdown
## Residual: <short-id>
Accepted: <why, and under what condition it is revisited>

<what is not covered>
```

For risks that belong to no single claim — typically a cross-cutting concern held as prose while
the steel thread runs. Labels come first here too: the grammar is uniform, so prose above a label
is a parse error rather than something silently swallowed.

## What never appears here

- **Test names as evidence for a claim.** Derived from `covers` tags.
- **The actual scope or quantification of an existing test.** Declared by the tag; the plan states
  what is *required*. `wrong-form` is the comparison.
- **A manual charter as if it were a result.** Only an imported, passed, unexpired execution receipt
  contributes evidence. Failed and expired receipts remain visible as holes.
- **Restatement of the claim.** The spec owns the predicate.
- **Anything true of every claim.** That belongs in `standards.md`.
