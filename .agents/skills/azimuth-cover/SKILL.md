---
name: azimuth-cover
description: Write evidence for an Azimuth claim and tag it honestly — pick the form the claim's own quantifier demands, build it, and record a deviation in the plan when that form is not available. Use when adding or changing a Covers tag, closing a hole reported by azimuth check, or rewriting evidence after a judgment.
---

# Writing evidence for a claim

A `covers` tag is a self-declaration. Nothing in the machine tier can tell whether it describes the
test: `azimuth check` compares the declared form to the required form and reports green when they
match, whether or not the test is what the tag says. The observed failure is not agents writing
weak tests — it is agents writing accurate-looking tags over tests that do not range over what the
claim ranges over, because the required form was expensive and the tag was cheap.

This skill is the authoring counterpart to `azimuth-verify`. Its output is a test plus a tag that
survives being read by someone who does not trust either.

## Before writing anything

1. **Read the claim** in the spec. Not the scenario id — the GIVEN/WHEN/THEN.
2. **Read the required form**: the plan entry for this claim if one exists, otherwise the project
   standard keyed by the requirement's criticality. Never assume a mapping; projects set their own,
   and a plan entry overrides it.
3. **Read the design entry** for the requirement, if there is one. It names the mechanism the claim
   rests on, and a test that does not exercise that mechanism is usually testing something else.

## The rule

**The claim names the axis. Range over that axis, not a convenient one.**

A quantifier in the WHEN is not decoration — it is the instruction for what must vary:

| The claim says | The axis is | Varying anything else proves nothing |
|---|---|---|
| "any further request" | arrival multiplicity | more request *shapes* |
| "has reached a terminal state" | the terminal set | more transitions into one terminal state |
| "delivered more than once" | delivery count | more payload variants |
| "an expired \<thing\>" | distance from the boundary | more amounts, more identifiers |
| "every \<site\>" | the set of sites | more behaviours at one site |

Evidence that varies an axis the claim does not quantify over is an example wearing a costume. It
is harder to catch than a plainly scripted test, so producing one is worse than not trying.

Count is not the criterion. Two cases derived from the domain can be universal; fifty authored ones
are an example set fifty times over.

## Four shapes

Pick by what the axis *is*.

### A. The axis is a finite set the system already knows

Derive the enumeration from the same source the system is built from — the state machine, the route
table, the container, the migration set. Never hand-list it: a hand-listed enumeration that misses a
member reports green over the gap, which is worse than no rule at all.

```
for each member of <enumeration derived from the system>:
    drive the system to that member
    assert the claim's predicate
```

The test then covers a new member the day one is added, and fails loudly if the enumeration cannot
be derived. That property — not the case count — is what makes it universal.

### B. The axis is a large or unbounded input space

Generate the input, and **compute the expected result from the generated input**. The mechanical
consequence is that no expected value in the test body can be a literal.

```
for each generated <input>:
    expected = <relation stated by the claim>(<input>)
    seed the system with <input>
    assert observed == expected
```

Then handle the boundary **explicitly**. Generation samples the interior; claims about thresholds
fail at the edge. If the claim has an ordered axis — a deadline, a limit, a cutoff — construct
just-inside and just-outside cases by hand and keep them beside the generated ones. Uniform sampling
inside a region cannot see a wrongly-placed edge.

### C. The axis is contention or ordering

Repeat the operation under real simultaneity, and assert the claim's predicate across the outcome
set rather than on one participant.

```
repeat <n> trials:
    fire <k> operations concurrently against one contended resource
    assert exactly the permitted number succeeded
    assert the store agrees
```

Two constraints. The contention must be real, which usually forces the scope up — a concurrency
claim at unit scope is vacuous, and the tag must say the scope the test actually has. And isolate
the mechanism: vary every *other* identifier so a neighbouring constraint cannot pass the test on
this one's behalf.

No test ranges over all interleavings. Repeated contention is accepted as universal because the
axis is right, not because the space is exhausted; say so in the plan if the trial count is thin.

### D. The axis is degenerate

Some claims have one case. If the WHEN fixes every free variable and the remaining variation is
irrelevant — clock time, generated identifiers — then `example` is the accurate tag and repeating
the test buys nothing. Declaring a higher form here is over-declaration, not caution: it costs the
reader the ability to tell which claims are actually ranged over.

## Self-check before the tag goes on

Four questions. Any "no" means the tag is not the one you were about to write.

1. **Is every expected value computed rather than written?** A literal the claim says should come
   from elsewhere means the test cannot distinguish deriving it from hard-coding it.
2. **Is the case set absent from the test file?** If the cases are typed into the test, the
   enumeration is yours, not the system's.
3. **If the domain gained a member tomorrow, would this test cover it?** If no, it is an example.
4. **Does the test construct the boundary, or only the interior?** Interior-only is not universal
   over an ordered axis.

The tag declares scope and oracle too, and they are held to the same standard: `component` on a test
that never touches the real store, or an oracle naming a source the test does not consult, is the
same failure as an over-declared quantification.

Keep the three relation-shaped oracles separate:

- `relational` checks a stated relation among values observed for one case;
- `metamorphic` compares executions connected by an intentional transformation without computing
  either absolute answer;
- `model-based` uses an independent model to compute the exact expected result for the input.

If a test contains several shapes, name the one that actually discriminates this claim. Comparing
a response total with its own breakdown is relational. Partitioning an input and comparing whole
and recombined executions is metamorphic. Repeating production logic in the expected-value path is
not model-based merely because it is in test code.

## When the required form is not available

This is the branch that decides whether the skill works. There are claims whose honest evidence
cannot reach the required form:

- the enumeration cannot be derived soundly, and hand-listing it would be worse;
- the axis exists but nothing can range over it — real elapsed time, third-party behaviour,
  interleavings beyond repetition;
- there is no oracle over the generated space, and inventing one means reimplementing the subject,
  so the test agrees with the code for the same reason the code is wrong;
- the claim spans a boundary the test's scope cannot reach — in which case the defect is scope, and
  quantification will not fix it.

**Do not resolve this by writing the required form on the tag.** Tag what the test is, and record
the deviation in the plan, where it is reviewable:

```markdown
## Claim: <scenario-id>
Quantification: example
Residual: <what is not ranged over, specifically>
Accepted: <why that is acceptable now, and what would change it>
```

A recorded gap with an author attached is a normal artifact. A tag that hides it is the one thing
this framework cannot detect from structure alone.

## Anti-patterns

- **Generating on the wrong axis.** The most damaging outcome, because it defeats a reader who
  checks only whether generation is present.
- **An oracle that reimplements the subject.** Passes for correct and incorrect implementations
  alike whenever both share the misunderstanding. Prefer a relation stated by the claim: within a
  case that is relational; across intentionally transformed executions it is metamorphic.
- **Hand-listed enumerations.** Report green over an unknown fraction of the domain.
- **Trial counts as reassurance.** Repetition without contention, or cases without variation, is
  the same evidence run repeatedly.
- **Over-declaring above the required floor.** Ladders mean a stronger form satisfies a weaker
  requirement, so this passes the check — and destroys the reader's ability to tell which tags
  mean anything.

## This skill's own falsifier

It exists to reduce the rate at which the agent tier returns `dishonest-tag`. Record the rate before
adopting it and after, over comparable claims. If skill-written evidence does not beat the baseline,
the skill is ceremony and should be deleted rather than expanded. Note the weakness of the
measurement if the same judge scores both sides.
