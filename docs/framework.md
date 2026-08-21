# Azimuth — what the framework is

Status: **derived**. This document states the current framework. [`decisions.md`](./decisions.md),
the format contracts and implementation are authoritative when this account disagrees with them.
Terminology is bounded by [`glossary.md`](./glossary.md).

---

## The central model

Azimuth is an **evidence control plane**. It keeps durable product intent, implementation linkage
and reviewed evidentiary meaning in a repository without making a test runner, CI system, analyzer
or monitoring provider authoritative for that meaning (D43).

A **Claim** is an addressable proposition about a product or its operation. Claims have two levels:

- a requirement-level Claim is a normative SHALL proposition and owns criticality; and
- a case-level Claim refines one observable condition and remains independently addressable.

The case is still written as a GIVEN/WHEN/THEN scenario, but it is not a second ontology beneath a
Claim. A result about one case does not silently establish its broader requirement.

Each Claim can have three facets (D3):

| Facet | Question | Repository authority |
|---|---|---|
| intent | What must be true, over what domain, and how much does it matter? | `spec.md` |
| mechanism | What makes it true? | `design.md` |
| evidence | Why should a particular method be believed for this Claim? | `verification.md` |

Criticality decides which facets are applicable (D20). A routine Claim stops at intent and owes no
realization, mechanism, Check, Evidence Binding or Qualification. Standard and critical Claims add
the applicable obligations. Every active Claim in this repository is routine during the
fast-moving alpha 2 phase. Its ordinary tests and release checks remain engineering safeguards;
they are not Azimuth evidence.

## Intent and mechanism

A spec is a named group of requirements with a declared, path-independent id. Every requirement
has a declared id, criticality and one or more case-level Claims. Case ids are unique per spec, so a
requirement can be split or merged without changing case identity. Specs are organized by problem
domain rather than by service because one Claim may be realized across several components and
languages.

The Claim predicate is prose. It has no machine-evaluable semantics, and Azimuth never infers that
an implementation is correct merely because it is linked.

A design entry declares a stable mechanism identity, enforcement kind and rationale, then resolves
it to one current artifact. Code can identify an implementation with
`implements-mechanism`; non-code artifacts can be bound through an extractor-resolved address. The
design identity survives deletion of its implementation, making the broken relation visible.

Residue belongs outside the Claim graph: orientation, danger zones, deliberately absent behavior
and other knowledge that cannot be derived. It may be recorded beside design, but it creates no
semantic relation.

## Repository evidence graph

The alpha 2 repository graph is deliberately sparse (D45):

```text
Check implementation --implements--> Check --Evidence Binding--> case-level Claim
                                             |
                                             +-- Qualification

Challenger --Challenge Plan--> exact Qualification fingerprint
```

### Check

A **Check** is a deliberately enrolled verification method with one atomic terminal proposition.
It is not every native test, analyzer rule or monitor in a repository. If two assertions can vary
independently, they are separate Checks even when one native process evaluates both.

Source code uses `ImplementsCheck(<project-global-check-id>)`. Extractors emit only the Check id,
resolved implementation site and exact source fingerprint. The marker names no Claim, evidence
form, context or Qualification. Several implementation sites may compose one Check. Unmarked tests
remain ordinary engineering tests and emit no Azimuth evidence linkage.

### Evidence Binding

An **Evidence Binding** is one reviewed Check-to-Claim edge. It explains why the Check's terminal
result bears on exactly one case-level Claim and declares:

- the binding proposition;
- actual `scope`, `quantification` and `oracle`;
- an exact required-context string map;
- the relations a Challenger may traverse; and
- one Qualification policy.

One Check may bind to several Claims only when the same atomic result honestly bears on every one.
Each relationship is a separate binding. Conversely, one Claim may receive several Checks with
different methods, contexts or oracles. A `(Check, Claim)` pair is unique.

Executable Checks demonstrate sampled behavior. Structural enforcement remains a mechanism and
can contribute to a future total assurance account without requiring a fictitious execution
result.

### Qualification

A **Qualification** is the reviewed decision that one exact Evidence Binding is credible under its
required context. Its id is the binding id, so each applicable edge has one current decision. A
Qualification is `qualified` or `rejected` and records its expected fingerprint, date, accountable
qualifier and rationale.

The expected value combines versioned fingerprints for the Check, binding and exact context.
Semantic source identity and source content participate; file paths, line numbers, mounts,
criticality and explanatory prose do not. This makes a relevant source, Claim, policy or context
change stale the precise decision without turning ordinary relocation into semantic drift.

A Qualification judges credibility. It neither records an execution nor establishes that its
Claim is satisfied.

### Challengers and Challenge Plans

A **Challenger** searches for a reason to distrust a Qualification. The proposition determines the
role, not the executable brand. Mutation testing, broad static analysis, flakiness repetition,
oracle mutation and qualification-oriented fault injection normally act as Challengers. A
claim-specific analyzer with an independent product oracle can instead implement a Check.

A **Challenge Plan** names one Challenger and semantic selectors. Current selectors resolve exact
Qualification fingerprints from bindings, Checks, realizations or mechanisms. Resolution unions,
sorts and deduplicates targets; a selector that resolves nothing is a Finding. Paths, globs and
line numbers are not semantic selectors, and zero matches never fall back to a whole suite.

Claim Judgment selectors are reserved in the format but cannot resolve until a separate change
defines a current total-composition Claim Judgment. There is no current Claim Judgment authoring
file or command.

A Challenger is not recursively qualified in alpha 2. Its quality is an ordinary tool-release,
conformance and review concern.

## Linkage and domains

Production code uses `realizes` to identify a site on a case-level Claim's realization path. The
relation is keyed by `(spec-id, case-id)` and carries no evidence form. A tagged site may be code or
declared delivery topology when routing is part of the behavior.

Routine Claims owe no realization linkage. For applicable non-routine Claims, several sites may
realize one Claim across components and languages. This fan-out is why the model must derive
traceability rather than maintain a second hand-written matrix.

A Claim can range over executions, a set of sites, a code artifact, paired derivations, aggregate
state over time or eventual absence (D13). A site-domain Claim names an independently derived
surface. Its enumerators must inspect the same source from which the system is built, such as a
route table, dependency container or type graph. A hand-maintained member list cannot establish a
universal domain.

Areas locate source participation. A surface can combine enumerator contributions from several
area mounts. An area realization obligation means that at least one realization must exist in each
named area; it is different from requiring every surface member to discharge a site-domain Claim.
Neither relation creates an evidence edge.

## Findings

`azimuth validate` deterministically reports **Findings**. Each Finding has one kind from the
exhaustive registry, a closed category, severity, source location, optional Claim and criticality,
detail and corrective help (D44). The categories are `intent`, `realization`, `verification`,
`mechanism`, `judgment`, `surface` and `execution`.

Findings include incomplete intent, dangling or missing realizations, unresolved mechanisms and
surfaces, invalid Check and binding cardinality, missing or stale Qualifications, unstable Check
implementations, verification applied to routine Claims, and unresolved Challenge Plans. The
machine tier establishes only that the repository account is structurally consistent; it cannot
establish product truth.

## Tool and derived outputs

The current top-level model commands are:

```text
azimuth validate
azimuth report traceability
azimuth export --out model.json
```

`azimuth validate` is the sole deterministic model validator. It does not execute Checks.
`azimuth report traceability` is a pure projection over selected case-level Claims, their ordered
realization identities and derived Check relationships. It creates no authored authority or
execution fact and writes no file unless `--out` is supplied.

`azimuth export` writes the complete derived model as format version 2. The export includes specs,
workspace data, realization and implementation linkage, mechanisms, Checks, Evidence Bindings,
Qualifications, Challengers, Challenge Plans and Findings. It contains no execution ledger data.
There is no assurance-specific export command in alpha 2.

The core reads language-neutral manifests rather than source. Ecosystem extractors emit the shared
version 2 linkage collections. This keeps source parsing outside the core and makes a language
integration an extractor concern instead of a fork of the model.

Nested change and project commands retain their bounded lifecycle meanings. They do not perform
model validation or Check execution.

## Multi-repository assembly

A project may be assembled from independent repositories without making paths global identity
(D33). The project catalog declares required repositories, stable areas and model-source
authorities. A workset supplies concrete revisions and content digests for repository manifests and
execution receipts.

Every federated source has semantic identity `(area, typed address)`. Repository, mount and path
are locators. Moving an unchanged area between repositories preserves linkage; splitting or
merging an area is an explicit identity transition. Model-source authority follows intent, so code
in one repository may realize a Claim owned by another without copying the Claim.

A local project result may be useful without being complete. Missing required inputs prevent a
complete assembly and cannot produce finalization. Project acceptance compares complete pre- and
post-archive accounts and verifies that one completed change moved unchanged within its singular
authority.

## Changes and archive

Current facets describe accepted state. A **change** proposes a target state through intent deltas,
solution design, implementation work and verification obligations. Proposed facts do not become
current simply because they appear in a change directory (D21).

Completion distils mechanisms that now exist and accepted intent into the current packages before
archiving the whole change. The archive retains alternatives, departures and the semantic history
of the transition. A change is not a branch, release or rollout; several repositories may
contribute work while one repository retains change authority.

Criticality changes through the same lifecycle without changing Claim identity. Raising a Claim
activates the applicable mechanism and evidence obligations. Lowering it records why those
obligations no longer apply and what would raise it again.

## Run exchange and deferred execution control plane

D46 implements the provider-neutral
[`azimuth-run-bundle` version 1](../azimuth/formats/run-bundle.md) exchange. One immutable bundle
revision accounts for one logical Run over one exact typed Subject and semantic plan. It freezes
planned and actual selection, physical activities, ordered attempts, one terminal Observation for
each actually selected Check and one terminal Challenge Result for each selected Challenger target.
Workspace, CI-candidate, artifact, deployment, service and bounded monitoring-window Subjects use
content or deployed-state fingerprints rather than mutable locators as their exact state.

Run bundles reduce retries and finite work units deterministically. Violations and challenge
findings survive retries; missing planned units and unfinished work cannot become positive results.
An omitted target creates no fabricated result. Full replacement corrections retain immutable
Subject, context, plan, source-execution and start anchors and form one fingerprint-linked chain.

The implemented service-free commands are:

```text
azimuth run verify --bundle <file> [--bundle <file> ...]
azimuth run inspect --bundle <file> [--bundle <file> ...] [--format text|json] [--out <file>]
```

Verification proves strict shape, internal identity, plan/actual agreement, reduction, references
and correction history. A protocol-consistent violation, challenge finding or partial Run is an
execution fact and therefore exits zero. Inspection presents a deterministic account and labels
current repository authority and Assurance State unresolved. Neither command calls a provider,
reads an artifact locator, writes to a service or treats protocol validity as product acceptance.

Generating plans from the current repository graph, resolving current decision targets for a Run,
translating native selectors, executing providers and importing native reports remain dependent
adapter and challenge-planning changes. `azimuth run plan`, `execute`, `import` and `ingest` are not
commands. The current repository Challenge Plan resolves its declared Qualification targets but
does not invoke a provider.

The optional Assurance Service is likewise awaiting the Run-ledger replacement. D42's version 1
claim-contract and project-snapshot wire remains isolated inside the existing service boundary
until that replacement removes it. It is not the alpha 2 repository model or Run-bundle protocol,
is not emitted by `azimuth export`, and receives no compatibility bridge. Authorization, durable
ingest, retention and Subject-specific Assurance State remain ledger work.

The authority split is current: repositories own Claims, Checks, Evidence Bindings and reviewed
decisions; Run producers own execution facts about exact Subjects. A standalone valid bundle does
not establish that its model or decision fingerprints are current.

Historical consumer feedback is retained only as an
[immutable provenance citation][historical-consumer-provenance].
The citation is documentary; no build, test, release or acceptance step reads that repository.

[historical-consumer-provenance]: https://github.com/drim-dev/azimuth-demo/blob/68a2eb5d46daf01ba087ec94b6a1ea7901c63bfd/azimuth/model/trips/rider-view/verification.md

## What is not claimed

Azimuth does not prove prose predicates, infer honest linkage from source, turn a clean Challenger
search into positive product evidence, or enroll native tests automatically. Its current outputs
are a versioned repository account, derived traceability and a standalone internally verified Run
exchange. Provider orchestration, durable ingestion and Subject-specific assurance remain deferred
rather than simulated through repository records or protocol validity.
