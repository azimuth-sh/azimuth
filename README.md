# Azimuth

> Traceability for code that agents write and humans don't read.

Azimuth is a method — and a small set of tools — for keeping software honest when an agent
writes most of the code. It answers one question mechanically: **is every specified behavior
implemented in code and checked by a test at the right kind of check?**

It is not a framework, a test runner, or a coverage tool. It sits *beside* your specs, code, and
tests and derives a **traceability matrix** from them, flagging the holes.

## Why

When a human writes code, the loop is: think → write → read your own diff → test. Reading is
the checkpoint. When an agent writes code, the middle two steps get cheap and fast, and the
reading checkpoint quietly disappears — nobody reads the whole diff. The risk shifts from
*writing* the code to *losing track of whether it does what was asked*.

The usual answers don't fill the gap:

- **Line coverage** tells you a line ran, not that a *specified behavior* is verified — and
  never that it's verified the *right way* (a completeness rule checked by one happy-path example
  is a hole coverage calls green).
- **Reading the tests** is the checkpoint that just disappeared.
- **Trusting the agent** is not a method.

Azimuth makes the checkpoint mechanical. The generative act (agent or human) **self-declares
intent** with a tag; a derived matrix **audits** the declarations against the spec. The tag is
the seam between *generating* code and *verifying* it.

## The model

Three artifacts, one derived from the other two.

### 1. Spec — the enumerated behavior

A spec (Azimuth reads [OpenSpec](https://github.com/Fission-AI/OpenSpec)) is a tree:

```
spec  →  requirement (a SHALL rule)  →  scenario (a WHEN/THEN behavior)
```

The **scenario** is the unit of coverage — the enumerated behavior. A requirement with five
scenarios where one is tested is a hole, and requirement-level thinking hides it. Each level
carries a **stable id** so links survive a rename of any display name, and each scenario carries
a **required form** — the honest *kind* of check it demands, as a pair of orthogonal axes:

```
scope          ∈ { unit, component, e2e }   — how much of the real system runs
quantification ∈ { example, invariant }      — one case (∃) vs a property over all (∀)
```

Both axes are ladders (a stronger form on either still satisfies a weaker requirement). A
completeness rule — a named invariant, "no implemented case is missing" — checked only by a unit
`example` is a hole on both axes, even though a test exists. An optional **oracle** label
(`direct, golden, metamorphic, model-based, contract`) records *how* the expected result was
obtained: descriptive only, never gated.

### 2. Linkage tags — intent, self-declared

Two tags, on the two sides:

- **`covers(spec, req, scenario, scope, quantification[, oracle])`** on a **test** — "this test
  verifies that scenario, at this form."
- **`realizes(spec, req, scenario)`** on **production code** — "this code site is on that
  scenario's path." No form: form is how a test *checks*, not a property of code.

The key is the stable **(spec-id, req-id, scenario-id)** triple. A test may carry several tags; a
scenario may be realized at several sites — which is exactly the cross-component fan-out (one
scenario realized in front-end, BFF, and backend becomes several matrix rows, one per site).

Tags are cheap to write and cheap to keep honest precisely because an agent writes them — the
same shift that removed the reading checkpoint pays for the annotation.

### 3. The matrix — derived, so it can't rot

Azimuth reads the spec's scenarios and the tags and produces, per scenario, its realizing code
sites and its covering tests, and flags the **holes**:

| Hole | Meaning |
|---|---|
| **uncovered** | a scenario no test covers |
| **unrealized** | a scenario no code realizes |
| **wrong-form** | covered, but never at the required form |
| **dangling tag** | a `covers` pointing at no scenario |
| **dangling realization** | a `realizes` pointing at no scenario |

Two independent axes — realized? (code) and covered? (test) — so the cross-states fall out
without double-reporting: *realized-but-untested* is an uncovered row that has code;
*tested-but-unimplemented* is an unrealized row that has tests; an *orphan* trips both.

## Two tiers: machine, then agent

Azimuth is deliberately split.

- **The machine tier** (these tools) is deterministic and cheap. It finds *structural* holes —
  uncovered, unrealized, wrong-form, dangling. It cannot be argued with and it seeds the next
  tier.
- **The agent tier** (a verify pass, on top) does *judgment* the machine can't: is a test
  *toothy* (does a mutation survive it?), is the tag *honest* (a test tagged `completeness` that
  is really an example), is a required behavior *missing from the spec itself*?

This split is load-bearing. A tag is only as honest as whoever wrote it — an agent can write a
toothless test and tag its form correctly. **The machine makes structure checkable; it does not
make truth checkable.** The agent tier is what keeps the self-declaration honest. Structure
without the judgment pass is a self-certification an agent can game.

## The code-map

Alongside the matrix, Azimuth keeps a **code-map**: a human-readable orientation of a component.
It has two halves, and only one survives automation:

- The **structural half** ("X realizes Y", "each feature has a store and a facade") is
  *derivable* — from the code and the `realizes` tags. Azimuth generates it; you don't hand-write
  it. This is why tests need no separate "test-map": self-describing artifacts don't get hand
  maps.
- The **judgment half** — danger zones, intentional broken corners, what is deliberately *not*
  here and why — is *not* derivable by anything. It is the durable core. No reflection recovers
  "this lost update is intentional, not a bug."

Derive the derivable; hand-hold only judgment.

## Tools

| Tool | Does |
|---|---|
| [`rtm`](./rtm) | reads specs + tags → the traceability matrix, exits non-zero on holes |
| [`code-map`](./rtm) | reads `realizes` tags → the derivable half of a code-map (per site, what it realizes) |

Both are single Rust binaries — logic lives once, language wrappers stay thin (the pattern of
ruff, biome, uv). Tags are read from a **language-neutral manifest** each codebase emits, or from
a comment convention the tools scan directly, so the method is polyglot: the spec is markdown,
the matrix is language-agnostic, and a scenario's fan-out can cross services and languages.

## Status

Early. `rtm` generates and checks the matrix and **dogfoods on Azimuth's own spec** (the `rtm`
tool is specified in `openspec/`, its Rust tests carry `covers` tags, and `rtm` run on itself
reports no holes). The method was extracted from a real codebase (the drim-dev challenge harness)
where it was proven on two capabilities before being generalized here.

## License

MIT.
