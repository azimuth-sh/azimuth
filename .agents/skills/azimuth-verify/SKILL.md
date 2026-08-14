---
name: azimuth-verify
description: Run the Azimuth agent tier over a claim or a spec — judge whether the covering evidence is toothy, whether the tags are honest, and whether the spec names what it should. Use when asked to verify, judge, or audit claims, or after adding tests that close holes.
---

# The agent tier

The machine tier makes structure checkable. It cannot make truth checkable: a tag is only as
honest as whoever wrote it, and an agent can write a toothless test and tag its form correctly.
This pass is what keeps the self-declaration honest, and without it a green matrix is a
self-certification.

Your output is a judgment per claim, recorded in the spec package's `judgments.md` under
`azimuth/model/<spec-id>/`.

## Worklist

```
azimuth judge --manifest <each manifest>
```

Each line is `spec  scenario  criticality  fingerprint  state`, followed by role-labelled inputs a
judgment must inspect: `realization`, `evidence`, `mechanism`, `challenge` and `context`. Judge everything
`unjudged` or `stale`; a `stale` judgment means the claim or one of those inputs changed since the
verdict, so the verdict no longer applies even if it still reads true.

`challenge` appears when mutation, static analysis or another judgment hardener has been imported.
Read the project policy in `azimuth/standards/judgment.md`, the named inputs and the report before
deciding the verdict. Outcomes and counts are a worklist, not a scorecard. `clean` does not mean the
claim is covered; `findings` does not become adverse until the finding is related to the predicate.

## What to examine, per claim

1. **Read the claim** in package `spec.md`, and the required form in sibling `verification.md` or
   the project standard.
2. **Read every realization site.** For each `Realizes` relation, state which part of the claim
   predicate the site establishes. Participation in the call path, transport of arbitrary data or
   use of a related mechanism is insufficient. A site that establishes no part of the predicate is
   a **dishonest-realization**.
   If the claim has area realization obligations, inspect every realization in every obligated
   area and state how that area contributes. One honest site elsewhere cannot excuse a missing or
   dishonest obligated realization. Do not demand an area-local test merely because the area is
   obligated. For a site-domain claim, inspect the surface declaration, every contribution witness
   and each discharge. Challenge whether the configured areas comprise the intended population;
   the enumerator proves membership only inside the accepted assignment.
3. **Read every covering evidence item.** For an automated test, read the body rather than its
   name. For an imported manual result, read the linked procedure, execution identity and
   observations rather than trusting `passed`. For detection evidence, read the metric producer,
   detector expression, detector test and silent-death account.
4. **Review every imported challenge.** Relate its findings and inconclusive outcomes to this
   claim's predicate and named subjects. For mutation, inspect survived, no-coverage, timeout,
   runtime-error and pending mutants: a relevant survivor demonstrates a plausible wrong
   implementation the selected evidence accepts, so the verdict is **toothless**. For static
   analysis, determine whether a finding contradicts the claim or realization; a clean broad scan
   is negative search, not proof. Never infer `sound` from a score or clean tool outcome.
5. **Check the design entry against the source.** A `Site:` is prose and nothing parses it, so it
   can name a type nobody wrote. Open the file it names before believing it. Three entries in this
   corpus described mechanisms that do not exist, and two verdicts were wrong because a judge cited
   one of them as evidence of a gap.
6. **Ask, in order:**
   - *Does every realization site establish part of this predicate?* If not, it is a
     **dishonest-realization**, even when another site makes the overall claim true.
   - *Would this evidence discriminate a plausible wrong system?* Construct one mentally: delete
     the guard, drop the constraint, return a constant, make the manual state ambiguous, or stop
     the metric producer. If the evidence still reads as passing, it is **toothless**.
   - *Does the evidence do what its declaration claims?* A tag saying `universal` on a test with
     one hard-coded case is a **dishonest-tag**, however true the case is. So is `component` on a
     test that never touches the real store, or `e2e` on a manual result executed against a mock.
     Detection is a claim about learning of a violation and must not be credited as proof that the
     production property holds.
   - *Does the claim describe the behaviour that matters?* If the code is right, the test is toothy,
     and a reader would still be surprised by something the spec never says, that is a **spec-gap**.
7. Only if none of those fire is the verdict **sound**.

## Traps

- **A passing test is not evidence of toothiness.** The question is never "does it pass".
- **Setup is not assertion.** A test that builds elaborate state and asserts one trivial
  consequence is usually toothless.
- **A test that never constructs the failure case is toothless**, even when the claim is true. A
  cancellation claim verified without ever cancelling anything is the common shape.
- **A manual charter is not a result.** Only an attributable, passed, unexpired receipt enters the
  model. If its linked case, procedure or observations cannot be inspected, the machine can
  establish attribution and freshness but this pass cannot responsibly call it toothy.
- **An alert name is not operational evidence.** Inspect the metric semantics, rule expression,
  threshold, injected violation and detector test. Ask whether a broken producer or muted route
  leaves the account quietly green. Repository bindings do not establish live scrape or
  notification health.
- **Mutation score is not a requirement and killed mutants are not product evidence.** Mutation
  testing probes the tests' ability to reject generated wrong implementations. It neither proves
  that the generated mutations are representative nor establishes the expected product result.
- **A clean broad scan is not blanket coverage.** It challenges the explicitly resolved sites in
  its bindings. It supplies no oracle for unrelated product predicates.
- **Do not turn realization review into general code review.** Judge whether the annotated site
  establishes the named predicate, not whether the implementation is otherwise correct. A correct
  implementation with a toothless test is still toothless; a true claim with an unrelated
  realization site still has a dishonest realization.
- **Do not soften a verdict because the code is yours.** The pass exists precisely for the case
  where the same author wrote the claim, the code, the test and the tag.

## Recording

```markdown
# Judgments: <spec-id>

## Claim: <scenario-id>
Verdict: sound | toothless | dishonest-tag | dishonest-realization | spec-gap
Fingerprint: <the fingerprint from `azimuth judge`, verbatim>
Judged: <YYYY-MM-DD>
Judge: <model or person>

What you examined, the wrong implementation you tried it against, and why the verdict. A verdict
without that is an opinion the next reader cannot check.
```

The fingerprint covers the claim's text, effective verification requirements, realization and
covering sites, the applicable design file and the source files of its machine bindings. Copy it
exactly: it is what makes the verdict expire when a relation is added, removed or moved, when an
examined source changes, or when the rest of the account changes. A judgment that cannot expire is
worse than none.

`sound` is not a pass mark to be handed out. If you find yourself writing `sound` for everything,
you are reading names rather than bodies.
