# azimuth

The core. Reads claims and linkage tags, derives a model, runs checks over it, and exports the
model for everything else to consume.

No dependencies (D17). `cargo build` needs nothing but a toolchain.

Install a checkout with `cargo install --path tools/azimuth`. The crate is release-shaped as
`azimuth` 0.1.0-alpha.1; `cargo package` verifies its standalone contents. Publication remains a
separate release operation after every selected artifact is qualified.

## Use

```
azimuth check                          # all checks, azimuth/model by default
azimuth check rtm --only 'billing/**'  # one check, scoped by id
azimuth export --out model.json
azimuth judge                          # claims with the fingerprint a judgment must carry
azimuth assurance export --project <id> --out assurance-snapshot.json
azimuth init                           # additive, idempotent project initialization
azimuth explore create <id> --title <text>
azimuth explore list
azimuth explore show <id>
azimuth change create <id> --title <text>
azimuth change list
azimuth change show <id>
azimuth change status <id>
azimuth change work-packages <id>
azimuth change instructions <id> --package <package-id>
azimuth change check azimuth/changes/<id>      # target projection and applied-state report
azimuth change finalize azimuth/changes/<id>   # gate completion and write finalization.json
azimuth change archive azimuth/changes/<id> --date YYYY-MM-DD
azimuth project check --project project.json --workset workset.json
azimuth project check --project project.json --workset workset.json --local experience
azimuth project locate --reference azimuth/project-reference.json
azimuth project observe --project project.json --repository experience --root . \
  --producer azimuth-emit-typescript/0.1.0 --manifest web.json --out experience.json
azimuth project finalize --project project.json --workset workset.json --out snapshot.json
azimuth project accept-change --project project.json --before active.json --after archived.json \
  --change <id> --date YYYY-MM-DD --out snapshot.json
```

Exit codes: `0` clean, `1` errors found, `2` the model could not be derived.

Selection operates on **ids**, not paths (`--only 'billing/**'`), so it keeps working if the tree
is reorganized. Ids in this file are illustrative: the tool knows nothing about any particular
corpus (D2).

Accepted artifacts default to sibling files discovered recursively under `azimuth/model/`:
`spec.md`, optional `design.md`, optional `verification.md` and optional `judgments.md`. The model
root is overridable with `--model`; evidence policy defaults to
`azimuth/standards/verification.md` and is overridable with `--standards`. Manifests are passed with
`--manifest`, repeatable. Local architectural declarations default to `workspace.json` beside the
model directory and are overridable with `--workspace`.

Agent-tier method policy lives separately at `azimuth/standards/judgment.md`. External executions
are imported as immutable observations with explicit claim bindings. Evidence bindings project
into ordinary `covers`; challenge bindings appear as `challenge` worklist inputs, never create
coverage and include their exact report, inputs and subjects in judgment freshness. The core runs
none of the native tools and contains no tool-specific result type.

Federated projects use a versioned project catalog plus a workset. Repository manifests carry
typed `(area, address)` source identity, observed Git revision, owned model-source digests and a
producer identity. Worksets pin their content digests. A complete assembly rejects missing inputs,
non-versioned model content, ownership conflicts, revision skew and composed receipts whose exact
subject set does not match the selected revision tuple. Repository-local references make the
owning catalog, areas and model sources discoverable without duplicating authority. A local
assembly is explicitly incomplete and cannot be finalized. The real-domain conformance trial
remains [immutable development provenance][multirepo-provenance]; federation tests in this
repository are synthetic and self-contained.

Repository observations derive their complete tracked change tree. Complete assembly rejects an
omitted or duplicate change authority. `project accept-change` verifies one content-preserving
active-to-archive move across complete pre-archive and post-archive worksets, including fresh exact
receipts, and rejects any other tracked edit in the archive revision. It emits the post-archive
snapshot. Git commits and external receipts remain integration inputs rather than tool side effects.

[multirepo-provenance]: https://github.com/drim-dev/azimuth-demo/tree/68a2eb5d46daf01ba087ec94b6a1ea7901c63bfd/experiments/multirepo

`rtm` is currently the only check. It is still one check among several by design (D9) — the check
set is per-project (D9.3), ids are a public interface (D9.1), and severity comes from criticality
rather than from the check (D9.2).

## What it does now

- **`spec.rs`** parses the format in `azimuth/formats/spec.md`. Strict: an unrecognized construct
  fails the parse with file, line and what was expected. A missing *declaration* is different—a
  requirement without `Criticality:` parses and becomes an `unclassified` hole (D6.2 vs D11).
- **`manifest.rs`** reads linkage and judgment-context manifests, keyed on the pair
  `(spec, scenario)` (D2.2). The
  alpha's triple is rejected with an explanation rather than silently accepted, so a stale emitter
  cannot produce tags that look fine and are not. Manifests also carry derived enumeration
  witnesses, compiler/schema artifacts, mechanism implementations, mechanism evidence and
  assurance observations.
- **`workspace.rs`** validates local areas, enumerator-backed surfaces and optional area realization
  obligations. Source area is derived from declared mounts rather than repeated in tags.
- **`assurance.rs`** projects a hole-free accepted model into one exact project snapshot and stable
  non-routine claim contracts for recurring assurance services. Contract fingerprints include
  claim, verification, surface and area-obligation semantics but exclude implementation bodies.
- **`plan.rs`** parses `azimuth/standards/verification.md` and sibling verification plans. Entries
  are deviations only—a claim with no entry is not unplanned, the standard applies.
  `Scope`/`Quantification`/`Oracle`
  state the *required* form; `Evidence` and its `Strength` declare a *provided* item, and
  `Strength` alone is an error because it reads as either.
- **`design.rs`** parses the mechanism facet. Entries key on the requirement — one index makes all
  three `captured-once` scenarios true, and recording it three times would be duplication. A
  requirement may carry several stable mechanism ids. Each resolves to exactly one emitted
  artifact through an explicit binding or one extractor-derived implementation site; `Expect:` can
  compare derived index properties. Strength is never written: it derives from the enforcement
  kind (D7). The `## Residue` section is read and never parsed.
- **`judgment.rs`** reads the agent tier's verdicts — `sound`, `toothless`, `dishonest-tag`,
  `dishonest-realization`, `spec-gap` — each carrying a fingerprint over everything the judgment
  looked at. Worklists distinguish realization sites from evidence and context.
- **`check.rs`** runs `rtm`.
- **`model.rs`** holds the derived model and writes the export (D10).
- **`change.rs`** projects additive and criticality-transition intent deltas, preflights accepted
  completion, fingerprints the derived model and gates deterministic archiving (D21.4, D24).
- **`federation.rs`** assembles revision-bound repository observations, enforces singular change
  authority and verifies accepted active-to-archive transitions (D33, D34).
- **`workflow.rs`** initializes a project, scaffolds and discovers changes and explorations, and
  validates dependency-ordered work packages with non-overlapping path ownership. It emits a
  portable worker instruction; native agent runtimes perform any actual delegation.

Four behaviours worth knowing:

- **Proof-strength evidence satisfies a demonstration requirement without a test** (D7). Strong
  enforcement is self-evidencing; the alpha's model reported that design as a violation.
- **Detection never satisfies a demonstration requirement.** "We'll alert on it" is the most common
  way a hard requirement is quietly downgraded, and it now fails rather than passing.
- **A plan cannot claim proof out of thin air.** `unbacked-proof` fires when proof-strength evidence
  has no mechanism at the top two rungs behind it. This is the first check needing all three
  artifacts, and it is the concrete argument that three beat one.
- **A judgment is evidence *about* evidence, and its value is negative** (D18, revising D14). It
  cannot make a claim covered; it can take a claim that looks covered and report it as a hole.
  Freshness isolates compiler-resolved evidence and realization sites while retaining whole-file
  fallback for inputs without a trustworthy boundary (D22, D28).

### Hole kinds

Thirty, in seven groups.

**Missing-facet** (D3's central structural claim — the facet is simply absent):

`unrealized`, `uncovered`, `dangling-tag`, `dangling-realization`, `dangling-plan-entry`,
`dangling-design-entry`, `wrong-form`.

**Incomplete-facet** — the facet is present but a required part of it is missing:

`unclassified`, `unaccepted-weakening`, `undeclared-mechanism`, `unjudged`.

These are the recorded falsifier for D3, which claims the taxonomy is generated by facet
*presence*. The premise fires and the conclusion does not: none of the four implies a fourth facet,
so the claim needs qualifying rather than replacing. D3 has not been amended.

Whether only these four count against the falsifier is unsettled — `unbacked-proof`, the agent-tier
kinds and the two site-class kinds are not missing-facet combinations either. See
`docs/framework.md`, which states the question without deciding it.

**Cross-facet consistency:** `unbacked-proof`, `unresolved-design-binding`,
`unresolved-evidence-binding`, `unresolved-detector-binding`, `enforcement-mismatch`,
`duplicate-observation`, `unresolved-observation-binding`.

**Mechanism linkage:** `dangling-mechanism-implementation`, `dangling-mechanism-cover`.

**Agent tier** — findings the machine tier structurally cannot reach, because a tag is only as
honest as whoever wrote it: `toothless-evidence`, `dishonest-tag-judged`, `spec-gap`,
`dishonest-realization`, `stale-judgment`.

**Site surface** — for claims ranging over a set of sites rather than over executions:
`missing-surface`, `unknown-surface`, `invariant-breach`,
`enumerator-unsound-or-underived`. An enumerator witness is
required before member findings are authoritative; tags never count as a complete enumeration.
`invariant-breach` is the one hole kind a per-scenario matrix structurally cannot find, because a
claim quantified over a set of sites is not established by evidence about one site however good
that evidence is.

**External evidence lifecycle:** `failed-evidence`, `expired-evidence`. An imported manual result
remains visible when adverse or stale and cannot be counted as coverage.

`undeclared-mechanism` is gated on the design artifact being in use at all. D8.1 requires each
mechanism to be usable alone, so a project running `rtm` without designs is not told that every
critical requirement is a hole. Partial adoption still reports.

## What it does not do yet

- **Change projection supports additions and criticality transitions.** Replacement, removal and
  scenario movement fail as unsupported rather than being approximated.
- **Symbol bindings establish existence only.** Database index bindings additionally compare
  uniqueness, columns and predicates. “Only caller,” transaction sharing and semantic properties
  still require a purpose-built analyzer, evidence or agent judgment.
- **Realization honesty is agent-judged, not inferred.** The machine supplies every realization
  source to the worklist and expires the verdict when its relation or source changes; it cannot
  decide whether arbitrary code establishes a prose predicate.
- **`invariant-breach` verifies only the weakest rung of the enforcement ladder** — a guard at every
  site. A choke point every member routes through would report N−1 breaches, which is exactly the
  defect D7 names in the alpha. Crediting one needs call-graph analysis in the extractor (D10.1).
- **Authored non-test evidence is taken on trust by the machine tier.** Imported manual receipts are
  checked for pass/fail and expiry; a prose attestation in a plan is still believed at its stated
  strength. Semantic honesty remains the agent tier's job, and nothing forces a judgment except
  `unjudged` on critical claims.
- **Area obligations establish participation, not evidence.** The machine derives each
  realization's area from its source locator and reports `missing-required-realization`; evidence
  continues to follow the verification plan rather than being duplicated per area.
- **Two domains of the six are exercised** (D13.3 closes the set at six). Claims are
  `(domain, predicate)`; the behavioural domain is what scenarios take implicitly, and the site
  class is declared with `## Invariant:`. The remaining four arrive as data, not as new artifact
  types.

## Tests

`cargo test`. Fixtures are synthetic by decision (D2): the moment this suite asserts against real
demo specs, the tool and the fixture are welded together and neither can move independently. Ids in
the fixtures are arbitrary strings and are not meant to name anything in the corpus.
