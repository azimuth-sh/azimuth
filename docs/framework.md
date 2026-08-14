# Azimuth — what the framework is

Status: **derived**. This document states the framework as it currently stands. It is assembled
from [`decisions.md`](./decisions.md), [`glossary.md`](./glossary.md), the three format contracts
and [`tools/azimuth/README.md`](../tools/azimuth/README.md); where it disagrees with any of them,
they win and this file is wrong. Terminology is bounded by the glossary.

It exists because those documents describe the framework by *decision* and by *facet*, and nothing
described it as a whole. A decision log records how a design was argued into existence, not what it
is now — several entries are marked *(revised)* or *(supersedes …)*, so reading it end to end gives
the history rather than the current state.

For whether any of this is *established*, see [`status.md`](./status.md). This document says what
the framework claims; that one says how much of it has survived contact with evidence.

---

## The central claim

**A requirement has three possible facets: what must be true, what makes it true, and how we know
(D3).** Recording only the first is what specification practice does today; recording the first and
third is what a traceability matrix does. The framework's bet is that the second is load-bearing
where assurance is required. Criticality sets that boundary: a routine claim deliberately stops at
intent, while standard and critical claims activate linkage and evidence (D20).

The consequence that makes this checkable rather than editorial: **holes begin with facets that are
missing relative to the declared rigor**, so a finding is a structural fact about the model rather
than a matter of taste (D3, D20). Other finding kinds qualify incomplete facets, cross-facet
consistency and the machinery that enumerates a claim's domain; D3's stronger taxonomy claim has
already been partially falsified.

---

## Five primitives

Everything else is structure layered over these, and D8 requires each layer to be removable without
breaking the core (§0.1 of `decisions.md`):

1. **Claims** — stable ids, criticality, the domain they range over (D13), and three facets (D3).
2. **Tags** — claim linkage (`realizes`, `covers`) and mechanism linkage
   (`implements-mechanism`, `covers-mechanism`).
3. **Evidence** — carrying strength and freshness (D4).
4. **The derived model** over the above, exported (D10).
5. **Changes** — the unit in which all of it moves (D11).

A claim is `(domain, predicate)`. The predicate is prose: it has no machine-checkable semantics,
and its truth is established by evidence rather than by evaluation. That is the largest gap between
this vocabulary and formal methods, and it is why no mechanism here claims to establish truth.

Every claim is universal. There is no quantifier field, because a constant field carries no
information (D13).

---

## The three facets

| Facet | Records | Lives in | Keys on |
|---|---|---|---|
| Intent | what must be true, over what domain, how much it matters | `spec.md` | scenario |
| Mechanism | what makes it true, and how strongly | `design.md` | requirement |
| Evidence | how we know, and how freshly | `verification.md` | scenario |

**Intent.** A spec is a named group of requirements; a requirement is one SHALL rule carrying
criticality; a scenario is a claim in GIVEN/WHEN/THEN form. Scenario ids are unique per spec rather
than per requirement, which is what makes splitting or merging a requirement free — scenarios move
between parents without touching a tag. Ids are declared in headings and never derived from paths,
so moving a package breaks nothing (`azimuth/formats/spec.md`).

**Mechanism.** An entry declares a stable mechanism identity, enforcement kind and rationale, then
resolves it to exactly one current artifact. A non-code artifact may be bound explicitly; a code
annotation normally lets its native extractor derive the binding. This is what makes deletion
visible: the design identity survives while the implementation edge disappears. Entries key on the
requirement because one index typically makes every scenario under it true at once.

**Evidence.** A plan records what *would be sufficient* to believe a claim, never what currently
exists — existing evidence is derived from `covers` tags, and hand-listing it would create a second
copy that drifts (D4.5). A claim with no plan entry is not unplanned; it means the project standard
applies unmodified.

**Residue** is the fourth thing in `design.md` and is deliberately outside the model: orientation,
danger zones, deliberately broken corners, what is absent and why. It participates in no check and
is derivable by nothing. It is named explicitly so the design file does not become a dumping
ground, and it is distinct from a verification *residual*, which records missing evidence. The
first is knowledge; the second is a gap.

**Accountability.** The model requires no roles (D3.1). Operating guidance calls the accountable
capabilities `intent owner`, `mechanism owner` and `evidence owner`; analyst or product, developer
and QA or quality engineer are a common mapping rather than a required organization. Ownership
means answering for a facet's sufficiency and freshness, not exclusive authorship. Test code may
be implemented by a developer, quality engineer or agent; the evidence owner must still be able to
judge whether it supports the claim, and ordinary engineering review still applies to the code
(D30).

**Physical layout.** The facets are logically separate and physically colocated (D32). Each spec
anchors a package at `azimuth/model/<spec-id>/spec.md`; optional siblings are `design.md`,
`verification.md` and `judgments.md`. Their declared spec ids remain authoritative and directory
proximity creates no semantic edge. Exact filenames make discovery closed, while optional files
keep routine intent and standard defaults lightweight.

---

## Changes and archive

The three facets describe accepted current state. A **change** proposes a target state: intent
deltas, solution design where needed, implementation work and verification obligations. The target
is the current model with those deltas applied; current checks do not treat planned facts as facts
about the running system (D21).

Change design and current design have different lifetimes. `azimuth/changes/<id>/design.md` may
name alternatives, components and mechanisms that do not exist yet. A current package's
`design.md` may name only mechanisms that were actually built and support accepted claims.
Completion distils the current facets from the result and archives the whole change—including
rejected alternatives, departures and work—as the semantic record of the transition.

A change is not a Git branch or a rollout (D31). Several short-lived work-package branches and
repositories may contribute to one semantic transition; one release may contain several accepted
changes. The current facets describe accepted behaviour of the codebase, not the percentage of a
production population already running it. Archive therefore normally follows engineering
acceptance and pre-production evidence. Deployment promotes an immutable mainline or established
release-candidate artifact through limited and wider exposure. Production observation delays
archive only when the proposal declared it necessary evidence before implementation.

Criticality changes through the same lifecycle without changing claim identity. A raise derives
new linkage, mechanism and evidence obligations; a lowering records why those obligations no longer
apply and what would raise the requirement again. The parsed protocol and its lifecycle commands are
in `azimuth/changes/README.md` (D21.4, D35).

### Exploration before a change

An exploration is a non-normative project account for research and decisions whose consequences
may span several changes (D36). It lives under `azimuth/explorations/<id>/`, can terminate without
a change, and never participates in current-model checks. Once its direction is confirmed,
individual change proposals carry the specific exploration decisions they implement. This keeps
uncertainty and rejected alternatives available without allowing them to masquerade as accepted
intent or current design.

The `azimuth-explore` skill owns the research/discussion boundary. Routine work whose intent is
already clear can still begin directly with a lightweight change.

---

## Linkage

Claim linkage has two tags, both keyed on `(spec-id, scenario-id)`:

- **`realizes`**, on production code: this site is on that claim's path. Carries no form, because
  form is how a test checks and not a property of code.
- **`covers`**, on a test: this test verifies that claim, at this *actual* scope and
  quantification. The plan states the *required* form; comparing the two is what produces
  `wrong-form`.

Both are declarations at the tagged site and required from the claim side only at `standard` and
`critical`. A routine claim owes neither. A test with no `covers` tag is an ordinary test outside
Azimuth's evidence model, not an exemption and not a hole (D20.1).

**Fan-out** — one claim realized at several sites, across components and languages — is the reason
specs are organized by domain area rather than by service. Mirroring services would duplicate every
cross-component claim.

Mechanism linkage is symmetric but has a different target:

- **`implements-mechanism`**, on production code, binds a compiler-resolved symbol to the stable
  `(design spec, mechanism id)` declaration. The design may instead carry an explicit `Binding:`
  for an extractor-resolved non-code artifact.
- **`covers-mechanism`**, on a test, records evidence for the mechanism's contract and actual form.

Mechanism evidence does not automatically fan out into claim coverage. A circuit-breaker state
machine may need one strong test suite, while each business claim still needs an honest account of
whether the breaker is applied over the relevant surface and whether that establishes the claim.

**Exemption** is a deliberate, attributable, reviewable opt-out from an obligation. An untagged
test claims no Azimuth evidence and therefore needs no exemption (D6.3, D20.1).

**Enumerator** — for a claim ranging over a set of sites, whatever produces the member set must be
derived from the same source the system is built from: the route table, the DI container, the type
graph. A hand-listed surface is worse than no rule, because it reproduces the bug the rule prevents
and reports green (D13.1).

**Areas, surfaces and realization obligations.** A local workspace declares durable areas and
their source mounts (D41). A surface binds one or more area mounts to independent enumerators; a
site-domain claim names the surface with `Over:` and requires every derived member to discharge the
claim. An ordinary standard or critical claim may instead require at least one realization in each
named area. The first is universal over surface members; the second is existential within each
architectural area. Neither creates per-area evidence obligations. Evidence form remains a property
of the verification plan, and semantic honesty remains an agent judgment.

---

## Evidence, and what is required

Evidence carries **strength**, and the ladder is `detection < demonstration < proof`:

- **Proof** — violation is unrepresentable. *Narrowing:* far weaker than the formal-methods sense.
  No obligation is discharged and no semantics is checked; the predicate is still prose. A unique
  index or a type constraint is proof-strength here because violation cannot be expressed, not
  because anything was proved.
- **Demonstration** — held for the executions sampled. Every test, including property tests: a
  wider sample is still a sample.
- **Detection** — we would learn if it stopped holding. A claim about the *detector*, never about
  the property, and every detection item needs a detector test proving it fires on an injected
  violation. Detector and detector test both bind to emitted artifacts (D4.3, D25).

**Scope** is `unit | component | e2e`, defined by what must be *real* rather than by how much runs
(D15). It applies to demonstration-strength evidence only: a static rule executes nothing and has
no scope. Defining it this way makes the rung partly machine-checkable — a harness knows whether it
started a database.

**Quantification** is `example | universal`: whether the evidence checked one case or ranges over
all of them. It is a property of evidence, not of the claim. The value was `invariant` until D19
renamed it, because a Floyd or Meyer invariant is a predicate about the *system* and this field
reports the breadth of the *evidence*. *Narrowing:* `universal` states the quantifier the evidence
ranges under, not exhaustiveness — a wider sample is still a sample.

**Oracle** describes where the expected result comes from. It is a closed vocabulary but not a
strength ladder, so the machine validates the name and never ranks or gates it:

| Oracle | Discriminating source |
|---|---|
| `direct` | An expected value written in the evidence |
| `golden` | A recorded prior output |
| `relational` | A stated relation among values observed for one case |
| `metamorphic` | A stated relation across executions connected by an intentional transformation |
| `model-based` | An exact expected result computed by an independent model |
| `contract` | An agreed interface or protocol contract |

The project standard (`azimuth/standards/verification.md`) maps criticality to required evidence
once, rather than per claim:

| Level | Strength | Quantification | Residual |
|---|---|---|---|
| `critical` | demonstration | universal | required |
| `standard` | demonstration | example | optional |
| `routine` | none | — | optional |

Default scope is `unit` for every claim, raised per claim where the claim's truth depends on
something real. Scope is deliberately *not* derived from criticality: an authorization rule can be
critical and honestly unit-checkable, while a `standard` claim about concurrent writes is vacuous
at unit scope. What determines scope is what the claim is about, not how much it matters.

Ladders mean a required form is a floor, not a target: proof satisfies a demonstration
requirement, and `universal` satisfies an `example` one.

---

## Mechanism, and why strength is never written

Enforcement kinds form a ladder (D7), strongest first:

| Rung | Kind | Violation is | Derived strength |
|---|---|---|---|
| 1 | `type`, `schema` | unrepresentable | proof |
| 2 | `constraint`, `choke-point` | rejected by storage, or routed through one place | proof |
| 3 | `middleware` | prevented where applied; application is opt-in | demonstration required |
| 4 | `guard` | checked at each site | demonstration required |

Strength is derived from the kind and never declared, because writing it would duplicate a
derivable fact. The top two rungs **are** proof-strength evidence — strong enforcement is
self-evidencing — which is why a claim enforced at rung 1 or 2 may carry a weaker evidence
requirement without that being a bargain.

The bottom rung is the design that leaks. "A guard at every site" is the weakest thing that can
still be called enforcement, and checking it means enumerating a set, which is where the machine
tier is weakest and D13.1's enumerator problem appears.

---

## Findings

Most hole kinds are missing-facet combinations, which is D3's central structural claim:

| Facets present | Hole |
|---|---|
| intent, no mechanism | `unrealized` |
| intent, no evidence | `uncovered` |
| evidence, no intent | `dangling-tag` |
| mechanism, no intent | `dangling-realization` |
| intent + evidence below the declared standard | `wrong-form` |

Four are **not** missing-facet: `unclassified`, `unaccepted-weakening`, `undeclared-mechanism` and
`unjudged` are *incomplete*-facet — the facet is present but a required part of it is missing. This
is recorded as a partial falsifier of D3: the premise fires, the conclusion does not, since none of
the four implies a fourth facet. D3 has not been amended.

Whether *only* these four count against the falsifier is unsettled. Read strictly, several other
kinds are also not missing-facet combinations — `unbacked-proof` is a cross-facet consistency
check, the agent-tier kinds qualify evidence rather than record its absence, and `invariant-breach`
and `unknown-surface` concern a claim's machinery, which the glossary already carves out for
`enumerator unsound or underived` (D13.2). The four above are the ones the source marks as
incomplete-facet in so many words. The wider reading would make D3's falsifier fire far harder, and
nothing has decided between them.

Two tiers produce findings:

- The **machine tier** is deterministic. It finds structural holes, cannot be argued with, and
  cannot establish truth.
- The **agent tier** judges what the machine cannot: whether every declared realization site
  establishes part of the predicate, whether evidence is toothy, whether its declared form is
  honest, and whether a required behaviour is missing from the spec. Its outputs audit the
  declared account and can withdraw trust; they never cover a claim (D14, revised by D18 and D28).
  Freshness follows
  compiler-resolved realization and evidence sites and conservatively falls back to complete files.
  External tools enter through one assurance-observation protocol (D39). An observation records an
  immutable execution once, then binds it to claims as evidence or as a challenge. Evidence
  bindings carry their own assertion and form and become ordinary `Covers` relations. Challenge
  bindings name resolved assurance subjects, create no evidence and can only sharpen the agent's
  review. Mutation and broad static analysis normally use the latter; explicit load and chaos
  oracles may use the former. Changed reports, inputs or subjects stale the affected verdicts.
  Recurring execution adds a second, optional boundary (D40): an agent qualifies the stable
  definition of what a successful run would establish, while CI and production systems append
  immutable observations for exact revisions, artifacts and deployments. Ordinary successful
  repetitions renew execution state without renewing semantic judgment or committing results to
  Git. Definition drift, failure, expiry, context or subject mismatch and current challenge
  findings close a lifecycle gate and create focused work. The definition retains the
  claim-specific semantics of D39's binding; the execution record references that definition
  rather than copying the interpretation on every run.
  A judgment whose inputs have changed is reported as
  `stale-judgment` rather than silently trusted — which is why a refactor invalidates prior
  verification by fingerprint rather than by anyone remembering.

The reference implementation under `services/assurance/` keeps that boundary optional. Its pure
Rust evaluator is used by both the original lifecycle experiment and an Axum/PostgreSQL service.
The service preserves idempotent immutable records, decision history, current gates, worklists and
repository-authored accepted-model snapshots; a Next.js client renders those projections without
evaluating them again. An accepted-model snapshot carries stable claim contracts for non-routine
claims (D42). Each contract fingerprints the claim, effective verification form and any surface or
area realization obligations. Implementations and enumerated members remain outside that contract:
their change creates a new exact execution subject, while unchanged semantics can reuse the prior
qualification. The CLI validates and projects this account; the service neither parses specs nor
runs enumerators.
Authentication, tenant isolation, signing, retention, report storage and production service
objectives are deliberately outside the reference slice. Routine claims acquire no service record
or lifecycle gate merely because the service exists.

---

## The tool

`azimuth` is the tool. `rtm` is one check among several, and the matrix is not the product (D9).

```
azimuth check                    # all checks
azimuth check rtm --only '…/**'  # one check, scoped by id
azimuth export --out model.json
azimuth assurance export --project <id> --out assurance-snapshot.json
```

Exit codes: `0` clean, `1` errors found, `2` the model could not be derived. Selection operates on
ids rather than paths, so it survives a reorganization. Severity comes from criticality, not from
the check (D9.2), and check ids are a public interface (D9.1).

The core is dependency-free (D17) and reads only **manifests**, never source. One extractor per
ecosystem finds tags in its own language and writes the same language-neutral manifest; that seam
is why adding a language is a day's work rather than a fork of the core. Extractors exist for .NET
and TypeScript.

The export is a first-class artifact (D10): checks, dashboards, PR annotations and the agent tier
are all consumers of it, and nothing re-parses specs.

### Multi-repository assembly

A project may be assembled from independent repositories without making paths global identity
(D33). The project catalog declares required repositories, stable areas, model-source authorities,
verification policy and composed receipts. A workset supplies concrete Git revisions and pins the
content digests of repository manifests and execution receipts.

Every federated source has identity `(area, typed address)`. Repository, mount and path are locators.
Moving an unchanged area between repositories therefore preserves linkage and judgment freshness;
splitting or merging an area is an explicit identity transition. Areas describe where source and
evidence originate, while specs remain organized by problem domain and `Scope: component` remains
an evidence form.

Model sources are federated by intent authority. Code in `rider-experience` may realize a
system-owned payments claim without copying that claim into an experience spec; experience-only
durable behaviour may be owned locally under `experience/**`. Duplicate spec ownership fails.

`azimuth project check` distinguishes a complete account from a useful local result. Missing
required inputs fail a complete assembly, while a local check reports project completeness as
unknown and cannot finalize. Execution receipts bind composed evidence to exact revision tuples,
and project finalization refuses partial or dirty worksets. A small repository-local project
reference locates the singular catalog and tells an entering agent its repository id; `project
locate` then reports the exact owned areas and model sources. The locator is duplicated, the
authority is not.

Project acceptance consumes the complete pre-archive and post-archive accounts in one
`project accept-change` operation (D34). Integration still creates the Git archive commit and runs
the post-archive evidence because Azimuth neither owns source-control policy nor manufactures
receipts. The command proves that one completed active change moved unchanged to one dated archive,
that unrelated revisions and source content did not move, and that both exact tuples are complete.
Its output is the post-archive project snapshot with the pre-archive revisions recorded.

**Machine-checkable design boundary.** Design entries bind to compiler/schema artifacts. The tool
confirms .NET symbol existence and compares migration-derived index uniqueness, ordered columns and
predicates. It does not infer “only caller,” shared transaction or semantic correctness from a
symbol. Non-test evidence remains trusted at its declared strength; that is the agent tier's job.
Crediting a choke point still needs call-graph analysis in the extractor (D10.1), so
`invariant-breach` verifies only the weakest rung of the ladder — a guard at every site.

**Broker boundary.** Delivery topology is a realization site when behavior crosses a broker (D26).
The machine can resolve the exchange, bindings, queues and dead-letter declarations present in
compiled code. A real-broker composed-stack test proves that those declarations compose. Neither
proves that an independently managed environment deployed the requested topology unchanged; that
requires a deployment-side enumerator.

---

## Decided, proposed, open

**Decided for this phase** — everything above, and reopenable by evidence from the fixture rather
than by argument. D20 makes routine claims intent-only and D21 restores changes and archives as the
transition around the three current-state facets.

**Experimental.** Additive changes are projected and accepted archives are automated after two
manual lifecycle observations. Other delta operations and rejected/abandoned archive automation
remain absent. A general typed realization graph remains a proposal: the route experiment showed
that derived surface membership does not imply semantic requirement discovery. D27's mechanism
identity has one product use; its application relation remains unresolved. D28 now makes declared
realization sites agent-auditable without claiming that the machine understands their semantics.
Multi-repository assembly is machine-tested through D33, including real Git histories and fault
injection; independent-team and cold-agent usability remain external validation work. D41's local
surface and area-obligation declarations are machine-tested; federated surface assembly and
additional enumerator kinds remain residual.

**Open.** Five of the seven questions recorded in `decisions.md` remain open — question 2 was closed
by D26 and question 3 by D15 — and they are open because they need evidence from the fixture, not
more argument: id semantics under split and merge; what `realizes` means for a rule with no site;
what is tagged when enforcement is a DB constraint; whether the six-domain set is right and should
stay closed (D13.3); how a generated check judges a domain whose members discharge differently.
The next mechanism experiment also asks whether cross-spec application needs reusable domain ids,
a mechanism catalog, or neither.

**Explicit non-goals for this phase** include backward compatibility, migrations and semver;
dashboards as deliverables (the export seam is the deliverable); and a configuration language for
rigor levels.

---

## What would falsify this

Recorded before the evidence existed. `status.md` holds the current results; two have fired.

| Falsifier | Status |
|---|---|
| >40% of requirements at top criticality → the level mechanism is theatre | **fired** (54%) |
| A hole kind that is not a missing-facet combination → D3 incomplete | **fired**, four times, and possibly harder — see above |
| The three role views over the export turn out identical → the facet split is decorative | never tested |
| Artifact and annotation cost exceeds what the defects justify → ceremony | never measured |
| The agent tier cannot reliably detect a dishonest tag → the core claim fails | **fired**: realization tags were absent from its worklist; D28 repairs the omission and the full pass removed fourteen unjustified relations, but self-review does not retroactively establish reliability |
| A concern fits none of the six domains → D13 wrong | holds; two domains exercised |

---

## Prior art, conceded

Traceability matrices, assurance cases, architecture conformance checking and mutation testing all
overlap this work, and the overlap is substantial rather than incidental. Traceability matrices
already link requirements to tests; assurance cases already record structured argument from claim
to evidence; conformance checking already compares an asserted architecture against code.

The claim to novelty is narrow, and only one part of it currently survives contact with evidence:
**a claim quantified over a set of sites is not established by evidence about one site, however
good that evidence is** — and per-scenario tracing structurally cannot notice the difference. That
is demonstrated once, by its author, in [immutable development provenance][site-proof]. Everything
beyond it is unmeasured.

[site-proof]: https://github.com/drim-dev/azimuth-demo/blob/68a2eb5d46daf01ba087ec94b6a1ea7901c63bfd/azimuth/model/trips/rider-view/verification.md

## What is not claimed

Nothing here establishes truth. The predicate is prose, `proof` means only that violation is
unrepresentable, and the agent tier judges the evidence without becoming evidence itself. The
framework's output is a structured account of what is claimed, what holds it up and how it is
known — together with a machine-checkable list of the places that account is incomplete.
