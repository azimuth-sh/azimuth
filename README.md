# Azimuth

**Azimuth is an evidence control plane for software.**

It answers one question about a codebase: **what do we guarantee, and why do we believe it?** The repository owns the durable answer: product intent, implementation linkage and the reviewed meaning of evidence. Test runners, CI systems, analyzers and monitoring providers can supply facts without becoming authoritative for what those facts establish.

A green test suite does not answer that question. It says some tests passed — not which requirements they bear on, not whether they would fail if the system were wrong, and not who decided any of that was enough. Azimuth keeps the answer next to the code, reviewed like the code, and derived rather than hand-maintained.

It is a command-line tool with no runtime dependencies and no hosted service. The canonical product site is <https://azimuth.sh>.

---

## The problem

Most teams have three accounts of the same system that drift apart: requirements in a tracker, tests in a repository, and an understanding in people's heads of which tests actually matter. The usual repair is a traceability matrix, maintained by hand, wrong within a quarter. Azimuth is not that artifact automated: a matrix records that a link exists, where Azimuth records why a method is believed, who decided so, and when that decision stopped being current.

Azimuth's position is that the matrix must be **derived**. You declare what must be true and mark the code that establishes it; the tool computes the rest and reports what is missing. Nothing is hand-listed, so nothing silently rots.

Its second position is that **evidence needs an argument, not a link**. That a test mentions a requirement proves nothing. Azimuth makes the connection explicit — this method bears on this requirement, for this stated reason, in this exact context — and makes it a reviewed decision with a name and a date attached.

## How it works

Modules organize Requirements. Scenarios describe expected behavior. Checks produce Results. The Azimuth Assurance Service (AAS) assesses the evidence and shows Requirement Status for a deployment.

Concepts arrive in the order you use them. Each step depends on the one before it, so nothing can be asserted before the thing it rests on exists.

### 1. Declare the Requirement and its Scenarios

A Requirement is behavior or a constraint the system must satisfy, governed independently. Under it
sit Scenarios — concrete situations and their expected outcomes — with free-form Markdown bodies,
independently addressable for evidence and execution. Requirements live in the specification of a
Module, a named hierarchical area that can contain nested Modules. Prose, tables, diagrams and
domain notations are all permitted; Azimuth preserves and fingerprints them without pretending to
understand their meaning. Ids are declared rather than derived from file paths, so reorganising
files changes nothing.

A Requirement may declare two optional review lines, `Realization: reviewed` and `Verification: reviewed`.
Scenarios inherit them and do not become separate governance centres. `Realization:` owes production
sites and an independent Realization Review; `Verification:` owes enrolled Checks for every Scenario and an
independent Verification Review.

> **A Requirement that declares neither stops here.** It is specified: no linkage, no evidence, no ceremony — one markdown entry, validated for internal consistency. This is the cost dial, and it is why adoption can start at effectively zero. Everything below applies only where you decided it should.

### 2. Mark the code that establishes it

Production code carries `Realizes(requirement-id)`. A site earns the marker when it establishes part
of the parent Requirement predicate—not merely because the request passes through it. Scenario
identity stays out of source markers.

Where a mechanism is what makes the Requirement true — a type, a schema constraint, a choke point — the optional realization facet names it and `ImplementsMechanism(mechanism-id)` binds the implementation. Extractors resolve both to semantic identities, so a rename or a file move is a locator change, not a semantic one.

### 3. Enroll a method as a Check

A Check is a deliberately enrolled verification method with one atomic outcome. Its implementations carry `ImplementsCheck(check-id)`. That is the whole annotation surface: three markers, and none of them carries evidence meaning.

Unmarked tests stay ordinary tests and assert nothing. Your existing suite does not become an accidental assurance claim.

### 4. Bind the Check to a Scenario

An Evidence Binding is the edge argument: why this atomic Check outcome bears on exactly one Scenario,
under exactly what Scenario-specific Execution Context. One Check may bind to several Scenarios only when its outcome
honestly bears on each, and each edge is a separate binding.

This is the step a traceability matrix skips.

### 5. Review the Requirement

Two independent reviews are global, and each Requirement owes only the ones it declares.

A **Verification Review** judges whether the verification is credible: the Checks, their oracles and
bindings, Scenario coverage, independence and whether the declared Challenges, present or absent, are
adequate. Its blocking dependencies are the Requirement text, Scenarios, Checks and bindings, Challenge
declarations and the execution units the Checks run.

A **Realization Review** judges whether the code at the realization sites does what the Scenarios say,
consistent with the realization facet where one exists. Its blocking dependencies are the Requirement
text, Scenarios and realization facet. Code changed at the sites since the reviewed commit is reported
as drift but never blocks: the review is an opinion about a code snapshot, not evidence.

The latest accepted review of each kind is stored in the model beside the spec, sealed, and written
only at finalization. Every review is attributed and fingerprinted, so a relevant change to a blocking
dependency marks the precise review stale and asks for it again. Paths, line numbers and prose do not.

### Alongside: execution and challenge

These are a separate plane, not later steps.

**Running enrolled work** produces execution facts (Check Results and Challenge Results) about an exact Verification Target — a source snapshot, an artifact, a deployment. Facts never author a decision, and a decision never records an execution.

**Verification Challenges** are optional searches, declared by the author, for a reason to distrust verification that already exists: mutation testing, broad static analysis, fault injection, an exploratory session. They can withdraw belief; they can never create it. A clean result records only that a configured search found nothing.

### Throughout: derived output

`azimuth validate` reports typed Findings over the whole account — dangling links, missing decisions, stale decisions, unsound enumerators. `azimuth report traceability` projects the graph. Neither is maintained by hand, and neither executes anything.

## What you write, and what the tool derives

| You write | The tool derives |
|---|---|
| Requirements, Scenarios and review declarations | which obligations apply |
| Three markers in source | semantic site identity and source fingerprints |
| Why a method bears on a Scenario | the traceability graph |
| Reviewed decisions with your name on them | which decisions have gone stale |
| Which objections must be searched for | what is missing, as typed Findings |

## Commands

```sh
azimuth init --agents codex      # scaffold the account and install Codex workflows
azimuth validate                  # deterministic Findings over the derived model
azimuth report traceability       # derived view of Requirements, linkage and decisions
azimuth export --out model.json   # the complete derived model for your own tooling
```

`azimuth validate` never executes anything. It reports whether the account is structurally honest — dangling links, missing decisions, stale decisions, unsound enumerators — and it cannot tell you whether the product is correct. Nothing in Azimuth claims otherwise.

For running enrolled work through a provider, `azimuth run plan | execute | import` drive explicitly configured short-lived adapter processes and return an immutable Run bundle. `azimuth run verify` and `azimuth run inspect` read those bundles without contacting anything.

## Adopting it

Start with every Requirement specified, declaring no review. That costs one spec file per area and gives you a declared, reviewable statement of what the system is supposed to do, validated for internal consistency.

Then declare `Verification: reviewed` or `Realization: reviewed` where the answer matters — a payment invariant, an authorization rule, a data-retention guarantee — and pay the evidence cost only there. Ten reviewed Requirements with real evidence are worth more than a thousand nominal ones, and a declaration only carries information if adding one is a deliberate act.

There is no hosted service to buy, no daemon to run, and no data leaves your repository.

## What Azimuth deliberately does not do

- It does not prove your prose. A Requirement predicate has no machine-evaluable semantics.
- It does not infer that code is correct because it is linked.
- It does not turn a clean scan, a passing suite or an absence of alerts into positive evidence.
- It does not enroll your existing tests automatically.
- It does not decide anything a person should decide. Every credibility judgment is authored and attributed.

These are design positions, not gaps. A tool that overclaims here is worse than no tool.

## What is not built yet

Stated plainly, because discovering it later is worse.

- **No runtime assurance state.** You cannot yet ask "what is assured in production right now." Durable Run ingestion, retention and per-Verification-Target Assurance State are deferred.
- **Decisions decay with content, not with time.** Staleness is fingerprint equality. There is no cadence, expiry or recurrence, so nothing re-opens a decision because the world moved.
- **Nothing caps review declarations.** No mechanism prevents every Requirement from declaring both reviews.
- **Azimuth does not yet apply its own evidence graph to itself.** No Requirement in this repository declares a review, so the graph is exercised by fixtures and conformance suites rather than by the framework's own assurance.

Open design questions are recorded under `azimuth/explorations/`, including these.

## Maturity

Alpha. Formats may change incompatibly between prereleases, and alpha 6 does not load an alpha 1 model. Evaluate it on a bounded pilot, not on a critical path you cannot revisit.

The published alpha 6 tag supports the Rust CLI and core including federation, the bundled consumer skills, templates, references and migrations, the .NET and TypeScript annotations and extractors, and the assurance API and diagnostic web images defined at that tag. Protocol and schema versions remain independent and are declared in the release catalog.

Qualified CLI targets are Linux x64, macOS ARM64 and Windows x64; qualified image platforms are Linux AMD64 and Linux ARM64. The C++, Go, JVM, Python and Rust annotation and extractor integrations, and everything under `experiments/`, are experimental source: CI exercises them, but `0.1.0-alpha.6` assigns them no package identity or support promise.

## This repository

Canonical development and release. Framework changes, pull requests and version history land here; publication derives artifacts from immutable version tags in the same history.

| Path | Holds |
|---|---|
| `contracts/` | public contracts: authoring formats, wire protocols, extraction, federation |
| `docs/` | derived framework account, glossary and operating guidance |
| `tools/azimuth/` | dependency-free Rust CLI and core, including federation |
| `tools/extractors/` | language and structural extractors |
| `packages/` | annotation packages; .NET and TypeScript are supported |
| `azimuth/model/` | this project's own Requirements — Azimuth described in Azimuth |
| `azimuth/changes/` | active and archived changes; the decision record |
| `azimuth/explorations/` | non-normative research and open questions |
| `experiments/` | self-contained conformance suites |
| `services/assurance/` | isolated alpha 1 service pending the Run-ledger replacement |
| `.agents/skills/` | agent workflows for exploration and change delivery |

Consumer-domain intent stays in the repository that owns it. This repository builds, tests and publishes without reading any consumer checkout; real-domain fixtures consume published versions and return findings here.

## Start here

- **Evaluating it** — this file, then `docs/framework.md` for the derived account of the model.
- **Using it** — `contracts/spec.md`, `contracts/realization.md` and `contracts/verification.md` for what you author; `contracts/markers.md` for the three source annotations; `tools/azimuth/README.md` for the commands.
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
