# Azimuth

**Azimuth answers one question about a codebase: what do we guarantee, and why do we believe it?**

A green test suite does not answer it. It says some tests passed — not which requirements they bear on, not whether they would fail if the system were wrong, and not who decided any of that was enough. Azimuth keeps the answer in the repository, next to the code, reviewed like the code, and derived rather than hand-maintained.

It is a command-line tool with no runtime dependencies and no hosted service. The canonical product site is <https://azimuth.sh>.

---

## The problem

Most teams have three accounts of the same system that drift apart: requirements in a tracker, tests in a repository, and an understanding in people's heads of which tests actually matter. The usual repair is a traceability matrix, maintained by hand, wrong within a quarter. Azimuth is not that artifact automated: a matrix records that a link exists, where Azimuth records why a method is believed, who decided so, and when that decision stopped being current.

Azimuth's position is that the matrix must be **derived**. You declare what must be true and mark the code that establishes it; the tool computes the rest and reports what is missing. Nothing is hand-listed, so nothing silently rots.

Its second position is that **evidence needs an argument, not a link**. That a test mentions a requirement proves nothing. Azimuth makes the connection explicit — this method bears on this requirement, for this stated reason, in this exact context — and makes it a reviewed decision with a name and a date attached.

## How it works

Concepts arrive in the order you use them. Each step depends on the one before it, so nothing can be asserted before the thing it rests on exists.

### 1. Declare the Claim and its Cases

A Claim is one independently governed SHALL proposition. Under it sit normative Cases, written as
GIVEN/WHEN/THEN clauses and independently addressable for evidence and execution. Ids are declared
rather than derived from file paths, so reorganising files changes nothing.

Every Claim declares its criticality: `critical`, `standard` or `routine`. Cases inherit it and do
not become separate governance centres.

> **Routine Claims stop here.** No linkage, no evidence, no ceremony — one markdown entry, validated for internal consistency. This is the cost dial, and it is why adoption can start at effectively zero. Everything below applies only where you decided it should.

### 2. Mark the code that establishes it

Production code carries `Realizes(spec, claim)`. A site earns the marker when it establishes part
of the parent Claim predicate—not merely because the request passes through it. Case identity stays
out of source markers.

Where a mechanism is what makes the Claim true — a type, a schema constraint, a choke point — the design facet names it and `ImplementsMechanism(spec, mechanism)` binds the implementation. Extractors resolve both to semantic identities, so a rename or a file move is a locator change, not a semantic one.

### 3. Enroll a method as a Check

A Check is a deliberately enrolled verification method with one atomic outcome. Its implementations carry `ImplementsCheck(check-id)`. That is the whole annotation surface: three markers, and none of them carries evidence meaning.

Unmarked tests stay ordinary tests and assert nothing. Your existing suite does not become an accidental assurance claim.

### 4. Bind the Check to a Case

An Evidence Binding is the edge argument: why this atomic Check outcome bears on exactly one Case,
under exactly what Case-specific context. One Check may bind to several Cases only when its outcome
honestly bears on each, and each edge is a separate binding.

This is the step a traceability matrix skips.

### 5. Qualify the method and decide applicability

A Method Qualification reviews the shared Check implementation, oracle, form and common context.
An Applicability Decision separately reviews whether that qualified method establishes one exact
Check-to-Case binding. Both are attributed and fingerprinted. Neither says that the Claim is
satisfied or that anything was executed.

### 6. Judge the Claim

A Claim Judgment is the decision over the parent Claim's total composition—every Case,
realization, mechanism, Evidence Binding, Method Qualification and Applicability Decision—with an
ordered basis and explicit residual risk.

All three decision kinds are fingerprinted, so a relevant change to the code, Case, Claim, context
or policy marks the precise dependent decision stale and asks for it again. Paths, line numbers and
prose do not.

### Alongside: execution and challenge

These are a separate plane, not later steps.

**Running enrolled work** produces execution facts about an exact subject — a workspace, an artifact, a deployment. Facts never author a decision, and a decision never records an execution.

**Challengers** search for a reason to distrust a decision that already exists: mutation testing, broad static analysis, fault injection, an exploratory session. They can withdraw belief; they can never create it. A clean result records only that a configured search found nothing.

### Throughout: derived output

`azimuth validate` reports typed Findings over the whole account — dangling links, missing decisions, stale decisions, unsound enumerators. `azimuth report traceability` projects the graph. Neither is maintained by hand, and neither executes anything.

## What you write, and what the tool derives

| You write | The tool derives |
|---|---|
| Claims, Cases and Claim criticality | which obligations apply |
| Three markers in source | semantic site identity and source fingerprints |
| Why a method bears on a Case | the traceability graph |
| Reviewed decisions with your name on them | which decisions have gone stale |
| Which objections must be searched for | what is missing, as typed Findings |

## Commands

```sh
azimuth init --agents codex      # scaffold the account and install Codex workflows
azimuth validate                  # deterministic Findings over the derived model
azimuth report traceability       # derived view of Claims, linkage and decisions
azimuth export --out model.json   # the complete derived model for your own tooling
```

`azimuth validate` never executes anything. It reports whether the account is structurally honest — dangling links, missing decisions, stale decisions, unsound enumerators — and it cannot tell you whether the product is correct. Nothing in Azimuth claims otherwise.

For running enrolled work through a provider, `azimuth run plan | execute | import` drive explicitly configured short-lived adapter processes and return an immutable Run bundle. `azimuth run verify` and `azimuth run inspect` read those bundles without contacting anything.

## Adopting it

Start with everything `routine`. That costs one spec file per area and gives you a declared, reviewable statement of what the system is supposed to do, validated for internal consistency.

Then raise criticality where the answer matters — a payment invariant, an authorization rule, a data-retention guarantee — and pay the evidence cost only there. Ten critical Claims with real evidence are worth more than a thousand nominal ones, and the levels only carry information if raising one is a deliberate act.

There is no hosted service to buy, no daemon to run, and no data leaves your repository.

## What Azimuth deliberately does not do

- It does not prove your prose. A Claim predicate has no machine-evaluable semantics.
- It does not infer that code is correct because it is linked.
- It does not turn a clean scan, a passing suite or an absence of alerts into positive evidence.
- It does not enroll your existing tests automatically.
- It does not decide anything a person should decide. Every credibility judgment is authored and attributed.

These are design positions, not gaps. A tool that overclaims here is worse than no tool.

## What is not built yet

Stated plainly, because discovering it later is worse.

- **No runtime assurance state.** You cannot yet ask "what is assured in production right now." Durable Run ingestion, retention and per-Subject Assurance State are deferred.
- **Decisions decay with content, not with time.** Staleness is fingerprint equality. There is no cadence, expiry or recurrence, so nothing re-opens a decision because the world moved.
- **Required evidence does not follow criticality.** Decision Policy is selected explicitly by
  each Method Qualification, Evidence Binding and Claim Judgment; criticality does not yet select
  one automatically.
- **Nothing caps criticality.** No mechanism prevents everything drifting to the top level.
- **Azimuth does not yet apply its own evidence graph to itself.** Every Claim in this repository is routine, so the graph is exercised by fixtures and conformance suites rather than by the framework's own assurance.

Open design questions are recorded under `azimuth/explorations/`, including these.

## Maturity

Alpha. Formats may change incompatibly between prereleases, and alpha 4 does not load an alpha 1 model. Evaluate it on a bounded pilot, not on a critical path you cannot revisit.

The published alpha 4 tag supports the Rust CLI and core including federation, the bundled consumer skills, templates, references and migrations, the .NET and TypeScript annotations and extractors, and the assurance API and diagnostic web images defined at that tag. Protocol and schema versions remain independent and are declared in the release catalog.

Qualified CLI targets are Linux x64, macOS ARM64 and Windows x64; qualified image platforms are Linux AMD64 and Linux ARM64. The C++, Go, JVM, Python and Rust annotation and extractor integrations, and everything under `experiments/`, are experimental source: CI exercises them, but `0.1.0-alpha.4` assigns them no package identity or support promise.

## This repository

Canonical development and release. Framework changes, pull requests and version history land here; publication derives artifacts from immutable version tags in the same history.

| Path | Holds |
|---|---|
| `contracts/` | public contracts: authoring formats, wire protocols, extraction, federation |
| `docs/` | derived framework account, glossary and operating guidance |
| `tools/azimuth/` | dependency-free Rust CLI and core, including federation |
| `tools/extractors/` | language and structural extractors |
| `packages/` | annotation packages; .NET and TypeScript are supported |
| `azimuth/model/` | this project's own Claims — Azimuth described in Azimuth |
| `azimuth/standards/` | this project's Decision Policies and Challenge Schedule |
| `azimuth/changes/` | active and archived changes; the decision record |
| `azimuth/explorations/` | non-normative research and open questions |
| `experiments/` | self-contained conformance suites |
| `services/assurance/` | isolated alpha 1 service pending the Run-ledger replacement |
| `.agents/skills/` | agent workflows for exploration and change delivery |

Consumer-domain intent stays in the repository that owns it. This repository builds, tests and publishes without reading any consumer checkout; real-domain fixtures consume published versions and return findings here.

## Start here

- **Evaluating it** — this file, then `docs/framework.md` for the derived account of the model.
- **Using it** — `contracts/spec.md`, `contracts/design.md` and `contracts/verification.md` for what you author; `contracts/markers.md` for the three source annotations; `tools/azimuth/README.md` for the commands.
- **Integrating it** — `contracts/manifest.md` to write an extractor, `contracts/adapter.md` with `contracts/run-bundle.md` to write a provider adapter, `contracts/export.md` to consume the model.
- **Changing it** — `docs/change-process.md`, then `AGENTS.md` for the authority order and working rules.

`contracts/` outranks the prose in `docs/` wherever they disagree, and the implementation and its tests outrank both.

Run the domain-independent repository checks with:

```sh
./scripts/check.sh
```

Some assurance integration tests require Docker. The script reports that boundary rather than treating an unavailable Docker daemon as a passing check.

## License

Apache-2.0. See `LICENSE`.
