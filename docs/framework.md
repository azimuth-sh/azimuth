# Azimuth — what the framework is

Status: **derived**. This document states the current framework. It is written from the artifacts and holds no authority of its own. When it disagrees with them, the authority order is the implementation and its tests, then the parser contracts in [`contracts/`](../contracts/), then [`azimuth/standards/verification.md`](../azimuth/standards/verification.md), then the accepted model under [`azimuth/model/`](../azimuth/model/). Terminology is bounded by [`glossary.md`](./glossary.md); each completed change keeps its own record under [`azimuth/changes/archive/`](../azimuth/changes/archive/).

---

## The central model

Azimuth is an **evidence control plane**. It keeps durable product intent, implementation linkage and reviewed evidentiary meaning in a repository without making a test runner, CI system, analyzer or monitoring provider authoritative for that meaning.

A **Claim** is an addressable proposition about a product or its operation. Claims have two levels:

- a requirement-level Claim is a normative SHALL proposition and owns criticality; and
- a case-level Claim refines one observable condition and remains independently addressable.

The case is still written as a GIVEN/WHEN/THEN scenario, but it is not a second ontology beneath a Claim. A result about one case does not silently establish its broader requirement.

Each Claim can have three facets:

| Facet | Question | Repository authority |
|---|---|---|
| intent | What must be true, over what domain, and how much does it matter? | `spec.md` |
| mechanism | What makes it true? | `design.md` |
| evidence | Why should a particular method be believed for this Claim? | `verification.md` |

Criticality decides which facets are applicable. A routine Claim stops at intent and owes no realization, mechanism, Check, Evidence Binding or Qualification. Standard and critical Claims add the applicable obligations. Every active Claim in this repository is routine during the fast-moving alpha 2 phase. Its ordinary tests and release checks remain engineering safeguards; they are not Azimuth evidence.

## Intent and mechanism

A spec is a named group of requirements with a declared, path-independent id. Every requirement has a declared id, criticality and one or more case-level Claims. Case ids are unique per spec, so a requirement can be split or merged without changing case identity. Specs are organized by problem domain rather than by service because one Claim may be realized across several components and languages.

The Claim predicate is prose. It has no machine-evaluable semantics, and Azimuth never infers that an implementation is correct merely because it is linked.

A design entry declares a stable mechanism identity, enforcement kind and rationale, then resolves it to one current artifact. Code can identify an implementation with `implements-mechanism`, from which the extractor derives the binding; a non-code artifact uses an explicit `Binding:` instead. The design identity survives deletion of its implementation, making the broken relation visible.

A marker-derived implementation is one atomic implementation-and-companion account. Its seven raw fields are `spec`, `mechanism`, `site`, `binding`, `file`, `lang` and `source_fingerprint`; its raw binding is exactly `<address-kind>:<site>`. Project assembly resolves the file to one area and atomically rewrites the implementation binding and companion Artifact id to `<area>|<address-kind>|<site>`. The compiler- or runtime-qualified site is semantic; the file is only an accountable locator. Local and federated assembly use the same rewrite, and neither file, mount, repository nor revision can disambiguate semantic identity.

The qualified-site account is deliberately ecosystem-specific. .NET uses namespace, declaring type, method and metadata parameter signature; Java and Kotlin use binary class, method and JVM descriptor. TypeScript and JavaScript use owning package, compiler module, receiver, symbol and canonical overload set. Go uses import path, receiver, function and `go/types` signature with positional generics. Python uses the one root-relative module and `__qualname__`. Rust binds one conventional Cargo target and reachable module to a normalized declared signature whose type-path spelling remains semantic. C++ accepts only a program-global, external-linkage, non-module, non-template and unconstrained declaration and uses qualified name plus canonical function type. Each emitter fails when that account is ambiguous rather than adding a path.

Residue belongs outside the Claim graph: orientation, danger zones, deliberately absent behavior and other knowledge that cannot be derived. It may be recorded beside design, but it creates no semantic relation.

The facets must also separate cleanly at N=1. If intent, mechanism and evidence only make sense when three different people author them, the separation is decorative and the framework cannot be dogfooded by a single maintainer. Ownership is therefore a removable layer rather than part of the model.

## Repository evidence graph

The alpha 2 repository graph is deliberately sparse:

```text
Check implementation --implements--> Check --Evidence Binding--> case-level Claim
                                             |
                                             +-- Qualification

case-level Claim --total composition--> Claim Judgment

Challenger --Challenge Plan--> exact Qualification or Claim Judgment fingerprint
```

### Check

A **Check** is a deliberately enrolled verification method with one atomic terminal proposition. It is not every native test, analyzer rule or monitor in a repository. If two assertions can vary independently, they are separate Checks even when one native process evaluates both.

Source code uses `ImplementsCheck(<project-global-check-id>)`. Extractors emit exactly the Check id, resolved implementation site, file locator, language and exact source fingerprint. The marker names no Claim, evidence form, context or Qualification. Several implementation sites may compose one Check. Unmarked tests remain ordinary engineering tests and emit no Azimuth evidence linkage.

### Evidence Binding

An **Evidence Binding** is one reviewed Check-to-Claim edge. It explains why the Check's terminal result bears on exactly one case-level Claim and declares:

- the binding proposition;
- actual `scope`, `quantification` and `oracle`;
- an exact required-context string map;
- the relations a Challenger may traverse; and
- one Decision Policy.

One Check may bind to several Claims only when the same atomic result honestly bears on every one. Each relationship is a separate binding. Conversely, one Claim may receive several Checks with different methods, contexts or oracles. A `(Check, Claim)` pair is unique.

Executable Checks demonstrate sampled behavior. Structural enforcement remains a mechanism and can contribute to a total-composition Claim Judgment without requiring a fictitious execution result.

### Qualification

A **Qualification** is the reviewed decision that one exact Evidence Binding is credible under its required context. Its id is the binding id, so each applicable edge has one current decision. A Qualification is `qualified` or `rejected` and records its expected fingerprint, date, accountable qualifier and rationale.

The expected value combines versioned fingerprints for the Check, binding and exact context. Semantic source identity and source content participate; file paths, line numbers, mounts, criticality and explanatory prose do not. This makes a relevant source, Claim, policy or context change stale the precise decision without turning ordinary relocation into semantic drift.

A Qualification judges credibility. It neither records an execution nor establishes that its Claim is satisfied.

### Claim Judgment and policy

A **Claim Judgment** is the repository-owned decision over one standard or critical case Claim's total applicable composition. Its exact identity binds the Claim and criticality, realizations, mechanisms and resolved artifacts, applicable surfaces and obligations, Evidence Bindings, recomputed Qualification expectations, one Decision Policy, verdict, ordered basis and residual risk. It is `accepted | rejected`; only a fingerprint-current accepted Judgment is executable. Routine Claims reject Judgments. A Run never authors or repairs one.

The strict `verification.md` block is:

```text
## Claim Judgment: <spec-id>#<case-id>
Verdict: accepted | rejected
Policy: <decision-policy-id>
Fingerprint: sha256:<64-lowercase-hex>
Judged: YYYY-MM-DD
Judge: <accountable identity>
Basis: <one or more ordered statements>
Residual risk: <one or more ordered statements>
```

Unknown labels, duplicate ids, routine targets, missing basis or residual risk and dangling Claims fail. Date, judge and rationale remain accountable metadata rather than fingerprint inputs.

One project `Decision Policy` namespace supplies the open Challenge forms required for Evidence Bindings and Claim Judgments. A separate `Challenge Schedule: current` assigns every required or declared form exactly once to `gate | scheduled`. Scheduling changes model, Plan and selection identity but not Qualification, Claim Judgment, policy or Challenger identity.

```text
## Decision Policy: <id>
Required challenge: <open-form>

## Challenge Schedule: current
Gate challenge: <open-form>
Scheduled challenge: <open-form>
```

Each policy has at least one distinct required form. The schedule is singular and non-empty; either lane may be empty. A form cannot occur in both lanes or remain unscheduled.

### Challengers and Challenge Plans

A **Challenger** searches for a reason to distrust a Qualification or Claim Judgment. The proposition determines the role, not the executable brand. It declares an open form and a non-empty set of required closed semantic scope kinds. Mutation testing, broad static analysis, flakiness repetition, oracle mutation and qualification-oriented fault injection normally act as Challengers. A claim-specific analyzer with an independent product oracle can instead implement a Check.

A **Challenge Plan** names one Challenger and uses exactly seven selector forms: Qualification from binding, Check, realization or mechanism, and Claim Judgment from Claim, realization or mechanism. Resolution retains every candidate as `selected`, `missing-decision`, `stale-decision`, `rejected-decision`, `invalid-decision`, `inapplicable` or `unresolved-relation`. Only a current positive decision is selected. One successful sibling never hides an adverse candidate, and zero selection never falls back to a path, glob or whole suite.

Executable selections deduplicate by exact Challenger and target identity. A challenged Qualification projects an impact edge to its owning Claim and current Claim Judgment; this graph edge does not fabricate a direct Judgment Challenge Result.

A Challenger is not recursively qualified in alpha 2. Its quality is an ordinary tool-release, conformance and review concern.

## Linkage and domains

Production code uses `realizes` to identify a site on a case-level Claim's realization path. The relation is keyed by `(spec-id, case-id)` and carries no evidence form. A tagged site may be code or declared delivery topology when routing is part of the behavior.

Routine Claims owe no realization linkage. For applicable non-routine Claims, several sites may realize one Claim across components and languages. This fan-out is why the model must derive traceability rather than maintain a second hand-written matrix.

A Claim ranges over one of exactly two domains: executions of a behavior, or a set of sites. A scenario takes the behavioral domain implicitly; an `Invariant` with an `Over:` surface takes the site domain. No other domain value exists, and the domain is never written in a spec. A site-domain Claim names an independently derived surface. Its enumerators must inspect the same source from which the system is built, such as a route table, dependency container or type graph. A hand-maintained member list cannot establish a universal domain.

Areas locate source participation. A surface can combine enumerator contributions from several area mounts. An area realization obligation means that at least one realization must exist in each named area; it is different from requiring every surface member to discharge a site-domain Claim. Neither relation creates an evidence edge.

## Findings

`azimuth validate` deterministically reports **Findings**. Each Finding has one kind from the exhaustive registry, a closed category, severity, source location, optional Claim and criticality, detail and corrective help. The exhaustive registry and its closed categories are in [`contracts/findings.md`](../contracts/findings.md). No current Finding kind maps to the execution category; it is reserved for the deferred execution plane.

Findings include incomplete intent, dangling or missing realizations, unresolved mechanisms and surfaces, invalid Check and binding cardinality, missing or stale Qualifications, unstable Check implementations, missing, stale or rejected Claim Judgments, incomplete Decision Policy or schedule coverage, verification applied to routine Claims, and unresolved Challenge Plans. The machine tier establishes only that the repository account is structurally consistent; it cannot establish product truth.

## Tool and derived outputs

The current top-level model commands are:

```text
azimuth validate
azimuth report traceability
azimuth export --out model.json
```

Authoring and project commands sit beside them: `azimuth init`, `azimuth explore create|list|show`, `azimuth change ...` and `azimuth project ...`. They scaffold and move artifacts and perform no model validation or Check execution.

`azimuth validate` is the sole deterministic model validator. It does not execute Checks. `azimuth report traceability` is a pure projection over selected case-level Claims, their ordered realization identities, derived Check relationships, Challenge resolution accounts and decision-impact edges. It creates no authored authority, direct Judgment Result or execution fact and writes no file unless `--out` is supplied.

`azimuth export` writes the complete derived model as format version 2. The export includes specs, workspace data, realization and implementation linkage, mechanisms, Checks, Evidence Bindings, Qualifications, Claim Judgments, Decision Policies, the Challenge Schedule, Challengers, Challenge Plans, candidate resolutions and Findings. Decision-impact edges belong to the traceability projection, not to the export. It contains no Run ledger data. There is no assurance-specific export command in alpha 2.

The core reads language-neutral manifests rather than source. Ecosystem extractors emit the shared version 2 linkage collections. This keeps source parsing outside the core and makes a language integration an extractor concern instead of a fork of the model.

Nested change and project commands retain their bounded lifecycle meanings. They do not perform model validation or Check execution.

## Multi-repository assembly

A project may be assembled from independent repositories without making paths global identity. The project catalog declares required repositories, stable areas and model-source authorities. A workset supplies concrete revisions and content digests for repository manifests and execution receipts.

Every federated source has semantic identity `(area, typed address)`. Repository, mount and path are locators. Moving an unchanged area between repositories preserves linkage; splitting or merging an area is an explicit identity transition. Model-source authority follows intent, so code in one repository may realize a Claim owned by another without copying the Claim.

A local project result may be useful without being complete. Missing required inputs prevent a complete assembly and cannot produce finalization. Project acceptance compares complete pre- and post-archive accounts and verifies that one completed change moved unchanged within its singular authority.

## Changes and archive

Current facets describe accepted state. A **change** proposes a target state through intent deltas, solution design, implementation work and verification obligations. Proposed facts do not become current simply because they appear in a change directory.

Completion distils mechanisms that now exist and accepted intent into the current packages before archiving the whole change. The archive retains alternatives, departures and the semantic history of the transition. A change is not a branch, release or rollout; several repositories may contribute work while one repository retains change authority.

Criticality changes through the same lifecycle without changing Claim identity. Raising a Claim activates the applicable mechanism and evidence obligations. Lowering it records why those obligations no longer apply and what would raise it again.

## Run and adapter execution control plane

Azimuth implements the provider-neutral [`azimuth-run-bundle` version 1](../contracts/run-bundle.md) exchange. One immutable bundle revision accounts for one logical Run over one exact typed Subject and semantic plan. It freezes planned and actual selection, physical activities, ordered attempts, one terminal Observation for each actually selected Check and one terminal Challenge Result for each selected Challenger target. Workspace, CI-candidate, artifact, deployment, service and bounded monitoring-window Subjects use content or deployed-state fingerprints rather than mutable locators as their exact state.

Run bundles reduce retries and finite work units deterministically. Violations and challenge findings survive retries; missing planned units and unfinished work cannot become positive results. An omitted target creates no fabricated result. Full replacement corrections retain immutable Subject, context, plan, source-execution and start anchors and form one fingerprint-linked chain.

The service-free protocol commands are:

```text
azimuth run verify --bundle <file> [--bundle <file> ...]
azimuth run inspect --bundle <file> [--bundle <file> ...] [--format text|json] [--out <file>]
```

Verification proves strict shape, internal identity, plan/actual agreement, reduction, references and correction history. A protocol-consistent violation, challenge finding or partial Run is an execution fact and therefore exits zero. Inspection presents a deterministic account and labels current repository authority and Assurance State unresolved. Neither command calls a provider, reads an artifact locator, writes to a service or treats protocol validity as product acceptance.

An explicit short-lived adapter boundary sits beside it. Strict `azimuth/adapters.json` configuration pins protocol and provider identity, executable and resource content, the description handshake, exact non-secret settings and environment literals, process limits and a capability dictionary. Core neither searches `PATH` nor invokes a shell. Its closed semantic capability classes are in [`contracts/adapter.md`](../contracts/adapter.md); provider families, configured `<adapter-id>/<capability-id>` addresses and Challenge forms remain open identities.

The reusable semantic Plan contains no provider route. A separate strict [`azimuth-run-launch-plan`](../contracts/run-launch-plan.md) version 1 binds the exact Subject, planned time, `execute | import` operation and complete semantic Plan to one configured adapter and one capability route for every selection. Route or configuration substitution changes the launch fingerprint and therefore the derived Run id.

The implemented provider journey is:

```text
azimuth adapter verify [--config <file>]
azimuth run plan --request <file> [--model <dir>] [--standards <file>] \
  [--workspace <file>] [--manifest <file>...] [--config <file>] [--out <file>]
azimuth run execute --plan <file> [--predecessor <bundle>...] \
  [--config <file>] [--out <file>]
azimuth run import --plan <file> --input <id>=<file>... \
  [--predecessor <bundle>...] [--config <file>] [--out <file>]
```

Verification compares the running adapter's complete description with configuration. Planning loads the complete unselected model and accepts strict Check-only, Challenge-only or mixed requests; the two arrays are required and their union is non-empty. A Check request names its exact configured capability and finite units. A Challenge request names an authored Plan, explicit capability, finite units and a nonzero candidate cap. Core derives fingerprints, targets, Challenger forms, lanes and scope; callers supply none of those semantic fields. There is no partial-model or `--only` mode.

The requested Challenge Plan union is fixed. Every reached candidate counts toward its Plan cap, including adverse dispositions, before cross-Plan selection deduplication. Any adverse candidate, cap overflow, mixed Qualification contexts or conflicting work fails planning. For every selected decision, the union must contain a runnable target for every form required by its Decision Policy; additional forms are strengthening. Core validates the exact `check.execute | check.import` or `challenge.execute | challenge.import` class, the Challenger's exact open form and the one-adapter boundary. It never chooses or adds a capability.

Each Challenge selection freezes its `gate | scheduled` lane and canonical semantic scope. Scope contains sorted selector anchors and complete decision inputs with typed identities and fingerprints. Each Challenge route projects exactly the source-backed scope items into accountable source, Artifact, enumeration or surface-member locator accounts. Scope changes semantic Plan identity; locator changes launch identity. An adapter translates this frozen account and never loads or reinterprets the repository model.

Execute and import stage the configured executable, resources and import inputs from the same open streams core hashes. Core clears the child environment and invokes the staged executable directly. On a supported host, process creation places it in a fresh process group before adapter code runs. One configured deadline bounds request writing, concurrent response and diagnostic reads and core's own wait. Core signals remaining group members on every terminal path and cleans their inherited pipes while they retain group membership. A host without that process-group primitive rejects the exchange before spawn as an exit-one transport failure.

This boundary is not non-escapable descendant containment. Authorized adapter code can call `setsid`, `setpgid` or an equivalent and leave the group. An escaped descendant cannot extend core's wait beyond the deadline, but core does not guarantee its termination. The stage and host controls provide neither daemon supervision nor hostile-code isolation and are not a filesystem or network sandbox.

The adapter returns one strict response and complete bundle. Core validates description, request, launch, routes, provenance, actual selection, reduction and bundle identity before atomic output. Repeatable predecessors must be one verified correction chain; the adapter receives its complete terminal account and must return revision zero or exactly the next revision correcting that terminal fingerprint. A valid violated Observation, Challenge finding, partial or cancelled Run, or adapter-returned protocol-valid `timed-out` Run fact is honest and exits zero. A host-enforced process deadline is a transport timeout and exits one, as does a semantic, identity, content or other transport mismatch. A `timed-out` Run is protocol-valid only when the adapter returns that complete fact within the host deadline; a host deadline produces no bundle. CLI and schema failure exits two. Neither nonzero class publishes an output.

Adapter, configuration, description, launch, capability routes, planned time and the complete normalizer join the correction anchors. Import-input identities remain protected in each revision but may change when later bytes from the same native execution arrive through the frozen route. A different adapter, capability or configuration therefore starts a different Run.

The current Run bundle version 1 requires this adapter provenance. It replaces the unpublished earlier shape in place, and no compatibility reader accepts it.

Generated plans currently represent Check and Challenge routes with model authority. Challenge Results are exactly `clean | findings | inconclusive`. Clean means only that the configured search found no objection; it creates no evidence or credibility. Every planned Challenge omitted from a partial, cancelled or timed-out Run has exactly one execution diagnostic scoped to its selection id and no fabricated Result. Scheduled omission is allowed deferral; gate omission records an honest execution failure. Added or substituted target, context, scope or units is a mismatch.

`model.extract` execution remains absent. Durable ingestion, authorization, retention and Subject-specific Assurance State remain Run-ledger work. Current planning defines no cache validity, cadence, historical-applicability or cross-Subject reuse semantics. Adapters remain bounded short-lived processes; there is no daemon, webhook, event gateway or long-running adapter boundary.

The synthetic [Challenge-planning conformance](../experiments/challenge-planning/README.md) uses only the public `validate`, `export`, `adapter verify`, `run plan`, `run execute`, `run import`, `run verify` and `run inspect` commands. It exercises all seven selectors, mixed planning, exact scope and routes, mutation, fault and broad-analysis meanings, scheduled omission, import provenance and selection mismatch without creating persistent state.

The optional Assurance Service is likewise awaiting the Run-ledger replacement. The alpha 1 claim-contract and project-snapshot wire remains isolated inside the existing service boundary until that replacement removes it. It is not the alpha 2 repository model or Run-bundle protocol, is not emitted by `azimuth export`, and receives no compatibility bridge. Authorization, durable ingest, retention and Subject-specific Assurance State remain ledger work.

The authority split is current: repositories own Claims, Checks, Evidence Bindings and reviewed decisions; Run producers own execution facts about exact Subjects. A standalone valid bundle does not establish that its model or decision fingerprints are current.

Historical consumer feedback is retained only as an [immutable provenance citation][historical-consumer-provenance]. The citation is documentary; no build, test, release or acceptance step reads that repository.

[historical-consumer-provenance]: https://github.com/drim-dev/azimuth-demo/blob/68a2eb5d46daf01ba087ec94b6a1ea7901c63bfd/azimuth/model/trips/rider-view/verification.md

## What is not claimed

One falsifier stays open and is cheap to run once the export carries non-routine content: if the intent, mechanism and evidence views over the export turn out to be substantially the same view, the three-facet split is decorative and the artifacts should collapse into fewer.

Azimuth does not prove prose predicates, infer honest linkage from source, turn a clean Challenger search into positive product evidence, create a repository decision from execution facts or enroll native tests automatically. Its current outputs are a versioned repository account, derived traceability and validated bounded adapter exchanges. Durable ingestion and Subject-specific assurance remain deferred rather than simulated through repository records or protocol validity.
