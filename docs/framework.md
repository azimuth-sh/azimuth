# Azimuth — what the framework is

Status: **derived**. This document states the current framework. It is written from the artifacts and holds no authority of its own. When it disagrees with them, the authority order is the implementation and its tests, then the parser contracts in [`contracts/`](../contracts/), then [`azimuth/standards/verification.md`](../azimuth/standards/verification.md), then the accepted model under [`azimuth/model/`](../azimuth/model/). Terminology is bounded by [`glossary.md`](./glossary.md); each completed change keeps its own record under [`azimuth/changes/archive/`](../azimuth/changes/archive/).

For the intended assurance control plane, harness boundaries, decision handling and production maintenance loop, see [intended-architecture.md](intended-architecture.md). That document describes direction rather than current executable contracts.

---

## The central model

Azimuth is an **evidence control plane**. It keeps durable product intent, implementation linkage and reviewed evidentiary meaning in a repository without making a test runner, CI system, analyzer or monitoring provider authoritative for that meaning.

Modules organize Requirements. Scenarios describe expected behavior. Checks produce Results. ASS assesses the evidence and shows Requirement Status for a deployment.

A **Requirement** is behavior or a constraint the system must satisfy: the independently governable
proposition about a product or its operation. It owns criticality, production realization,
Requirement Review and exact-target Requirement Status. A **Scenario** is a concrete situation and
its expected outcome inside that Requirement's predicate. Scenarios are addressable for evidence,
Runs, Check Results and Verification Challenge impact, but own none of the parent governance. A
result about one Scenario does not silently establish its broader Requirement. A **Module** is a
named, hierarchical area of the specification that owns Requirements and nested Modules; it implies
no code package, service, repository or deployment boundary.

Each Requirement can have three facets:

| Facet | Question | Repository authority |
|---|---|---|
| intent | What must be true, over what domain, and how much does it matter? | `spec.md` |
| mechanism | What makes it true? | `design.md` |
| evidence | Why should a particular method be believed for each Scenario? | `verification.md` |

Criticality decides which facets are applicable. A routine Requirement stops at intent and owes no
realization, mechanism, Evidence Binding or evidence review. Standard and critical Requirements add the applicable obligations. Every active Requirement in this
repository is routine during the fast-moving alpha phase. Its ordinary tests and release checks
remain engineering safeguards; they are not Azimuth evidence.

## Intent and mechanism

A spec is the intent document of one Module, with a declared, path-independent id. Every Requirement has a declared
id, criticality and one or more Scenarios. Requirement and Scenario IDs are stable project-wide lower-kebab names in separate typed namespaces. Module membership and Scenario parent relationships are mutable and separately declared. Specs are organized by problem domain rather than by service because
one Requirement may be realized across several components and languages.

Requirement and Scenario bodies are non-empty free-form normative Markdown. They may use prose, tables,
diagrams, code fences or a domain notation in any natural language. Core preserves their content
and includes it in semantic fingerprints, but does not interpret natural-language keywords or
notation. Structural validation establishes identity and assurance relationships; human and agent
review owns clarity, consistency and truth. Azimuth never infers that an implementation is correct
merely because it is linked.

A design entry declares a stable mechanism identity, enforcement kind and rationale, then resolves to either one explicitly bound artifact or a set of distinct tagged source sites. Code identifies each participating site with `implements-mechanism`; a non-code artifact uses an explicit `Binding:` instead. The design identity survives deletion of all implementations, making the broken relation visible. Extraction cannot establish whether the tagged set is complete or adequate for the rationale.

A marker-derived implementation is one atomic implementation-and-companion account. Its six raw fields are `mechanism`, `site`, `binding`, `file`, `lang` and `source_fingerprint`; its raw binding is exactly `<address-kind>:<site>`. Project assembly resolves the file to one area and atomically rewrites the implementation binding and companion Artifact id to `<area>|<address-kind>|<site>`. The compiler- or runtime-qualified site is semantic; the file is only an accountable locator. Local and federated assembly use the same rewrite, and neither file, mount, repository nor revision can disambiguate semantic identity.

The qualified-site account is deliberately ecosystem-specific. .NET uses namespace, declaring type, method and metadata parameter signature; Java and Kotlin use binary class, method and JVM descriptor. TypeScript and JavaScript use owning package, compiler module, receiver, symbol and canonical overload set. Compiler default-library names require a resolved symbol anchored in those libraries; consumer homonyms fail. Union members use sorted `union(...)` accounts, including within nested types, so qualified sites contain no `|`. Go uses import path, receiver, function and `go/types` signature with positional generics. Python uses the one root-relative module and `__qualname__`. Rust binds one conventional Cargo target and reachable module to a normalized declared signature whose type-path spelling remains semantic. C++ accepts only a program-global, external-linkage, non-module, non-template and unconstrained declaration and uses qualified name plus canonical function type. Each emitter fails when that account is ambiguous rather than adding a path.

Residue belongs outside the Requirement graph: orientation, danger zones, deliberately absent behavior and other knowledge that cannot be derived. It may be recorded beside design, but it creates no semantic relation.

The facets must also separate cleanly at N=1. If intent, mechanism and evidence only make sense when three different people author them, the separation is decorative and the framework cannot be dogfooded by a single maintainer. Ownership is therefore a removable layer rather than part of the model.

## Compact verification account

The compact verification account separates authored meaning, machine execution and independent review. `verification.md` owns Check method prose and inline Evidence Bindings. The typed Check–Scenario pair identifies each binding, and its Contribution describes what that method may establish for that Scenario. Source markers identify implementations; they cannot author binding meaning or approve a review. Compact method prose is retained as prose, without a synthesized `Terminal` field.

Explicit packages add shared concepts without changing Requirement kinds. `azimuth.surface` supplies Surfaces, independent Expectation Sets, typed producer descriptors and deterministic report validation. `azimuth.network` supplies named Probes and origins. .NET attributes and Python annotations use their package namespaces. A Surface publishes field names; its emitter derives language types and supported values from the enumerator's output. `azimuth surface verify` checks exact Verification Targets, producer support, completion, schema, unique keys, membership fingerprints and joins in both directions. It does not establish runtime behavior from registration metadata.

Requirement Review assesses whether the proposed verification is a sensible way to verify the Requirement and whether it has been implemented correctly, over the complete authored Requirement account: Scenarios, design, realizations, Check methods and implementations, oracles, coverage, Verification Challenges and limitations. It records `accepted | rejected` against exact definition dependencies, without an execution Verification Target. A Requirement Status record separately assesses Run Check Results for one exact Verification Target against the current reviewed Requirement and records `supported | violated | unresolved` with residual risks. Deployment Requirement Status is computed in ASS from applicable evidence and presented as Satisfied, Not satisfied or Unresolved; see [deployment-requirement-status.md](../contracts/deployment-requirement-status.md). Execution and structural validation author neither assessment. See [assurance-reviews.md](../contracts/assurance-reviews.md).

No separate Check Review, Method Qualification, Applicability Decision or Requirement Judgment stage is mandatory in this compact account. Missing, rejected, stale, invalid and ambiguous dependencies remain visible. Changing definitions requires reassessment; rerunning unchanged Checks does not require repeating their definition review.

`azimuth assurance authority` exports a complete unselected authority projection whose model identity matches Run planning. Trusted CI publishes it for each commit that reaches main, with its committed reviews and a ratification verdict, in the project-scoped Run ledger; the latest published main model is the default model and no owner selects or accepts it. The ledger retains Runs, correction chains and independently authenticated reviews, then computes exact-Verification-Target state against the default model. See [run-ledger.md](../contracts/run-ledger.md).

## Review and execution relationships

```text
Requirement + Scenarios + design + sources + verification -> Requirement Review
Check implementation -> Check --Contribution--> Scenario --> Requirement
Check execution --exact Verification Target--> Check Result
Current Requirement Review + applicable Check Results -> Requirement Status
Verification Challenge --exact assessment target--> Challenge Result
```

A Check is deliberately enrolled; it is not every test, analyzer rule or monitor in a repository. One implementation may support several Checks, and one Check may contribute to several Scenarios when its assertions bear on those Scenarios. `ImplementsCheck(<check-id>)` identifies the implementation, not evidentiary meaning or reviewer approval.

An inline Evidence Binding explains what a Check establishes for a Scenario. The typed Check–Scenario pair identifies that relation. Definition review assesses its credibility as part of Requirement Review; observed support is assessed as part of Requirement Status. Structural mechanisms can support the reviewed design without inventing Check Results.

A machine-executed Verification Challenge searches for reasons to distrust an exact Requirement Review or Requirement Status assessment. Its declaration lives beside Checks in `verification.md`; code or configuration supplies its implementation. Mutation, adversarial requests and static analysis may serve this role. The proposition being investigated determines the role: a bounded product assertion is a Check, whereas a search for blind spots in an assessment is a Verification Challenge.

Source scope is selected through meaningful Requirement realization and Mechanism implementation tags. Ecosystem emitters resolve semantic sites and accountable locations; adapters translate them into tool filters. Target scope is distinct from compilation/analysis dependencies and exercised Check scope. Missing mappings, exclusions and tool-driven expansion remain visible in the scope report. Tags do not prove completeness.

Planning selects an exact assessment fingerprint. The execution Verification Target is separate from the assessment target. Current Stryker integration pins exact source input for an isolated mutation batch and retains native statuses and locations; materialized per-mutant Verification Targets and mutant-specific product Check Results are unavailable. Future captured mutants would require distinct Verification Targets before their Check Results could support Requirement Status. Results are `clean | findings | inconclusive`; clean is only a negative search fact. Findings require disposition and, when relevant, correction and independent reassessment. There is no aggregate mutation-score assurance or recursive Verification Challenge qualification stage.

The earlier repository declaration lane remains separately documented in [verification.md](../contracts/verification.md), and earlier independent review records retain [their historical contract](../contracts/history/assurance-reviews-before-claim-health.md). Historical decisions do not substitute for current Requirement Review or Requirement Status.

## Linkage and domains

Production code uses `realizes` to identify a site on a parent Requirement's realization path. The
relation names the project-global Requirement ID and carries no Scenario or evidence form. A tagged site may
be code or declared delivery topology when routing is part of the behavior.

Routine Requirements owe no realization linkage. For applicable non-routine Requirements, several sites may realize one Requirement across components and languages. This fan-out is why the model must derive traceability rather than maintain a second hand-written matrix.

A Requirement ranges over one of exactly two domains: executions of a behavior, or a set of sites. A
Requirement with authored Scenarios takes the behavioral domain implicitly; an `Invariant` with an `Over:`
surface takes the site domain and receives one implicit Scenario. No other domain value exists.

Areas locate source participation. A surface can combine enumerator contributions from several area mounts. An area realization obligation means that at least one realization must exist in each named area; it is different from requiring every surface member to discharge a site-domain Requirement. Neither relation creates an evidence edge.

## Findings

`azimuth validate` deterministically reports **Findings**. Each Finding has one kind from the exhaustive registry, a closed category, severity, source location, optional Requirement and criticality, detail and corrective help. The exhaustive registry and its closed categories are in [`contracts/findings.md`](../contracts/findings.md). No current Finding kind maps to the execution category; it is reserved for the deferred execution plane.

Findings include incomplete intent, dangling or missing realizations, unresolved mechanisms and surfaces, invalid Check relationships, unsupported or unstable implementations and unresolved Challenge scope. Change validation distinguishes structural Findings from acceptance obligations. Exact review currentness and observed Requirement Status remain separate assessments; structural validity is not evidence of a supported Requirement.

## Consumer installation and guidance

The CLI embeds a release-owned consumer resource cohort independently of this repository's contributor skills. `azimuth init` requires an explicit `codex`, `claude`, combined or `none` agent choice, installs only the selected managed copies, and writes `azimuth/installation.json` with release, migration-line, integration, component, path and digest accounts. A pre-existing relative `.claude/skills` alias to the repository's `.agents/skills` directory may be adopted only by explicit request; Azimuth never creates a symlink.

Installed skills own stage sequence, gates and stopping conditions. Templates own initial shape. `azimuth reference list|show` exposes the running parser's version-matched authoring descriptors and bundled prose without copying a reference library into the repository. `azimuth change brief` is different: it renders a contextual handoff for one eligible work package, so the retired `change instructions` name has no compatibility alias.

Agent additions and removals update only accounted workflow files. Components are installed through their native ecosystem first, then registered explicitly with a repository-contained manifest; Azimuth validates the supported identity and exact running release but never edits the manifest or invokes its package manager. `azimuth update --check|--dry-run` and `azimuth update` compare the complete managed cohort with the running CLI, fail on modified resources, alias drift or component drift, and perform no network access, self-upgrade or semantic account rewrite.

Account migration is a separate boundary. `azimuth migrate plan` uses only a dedicated historical reader, classifies the complete transition as automatic, review-required or unsupported, and binds it to exact input content. `azimuth migrate apply` refuses drift, unresolved semantic review and unknown edit schemas. Historical forms do not become acceptable to normal validation. Migration edges accumulate only within the catalog's explicit `migrationLine`; crossing a line is an explicit no-migration boundary unless a later release declares otherwise.

## Tool and derived outputs

The current top-level model commands are:

```text
azimuth validate
azimuth report traceability
azimuth export --out model.json
```

Authoring, installation and project commands sit beside them: `azimuth init`, `azimuth reference ...`, `azimuth agent ...`, `azimuth component ...`, `azimuth update`, `azimuth migrate ...`, `azimuth explore create|list|show|archive`, `azimuth change ...` and `azimuth project ...`. They do not execute Checks.

`azimuth validate` is the sole deterministic model validator. It does not execute Checks. `azimuth
report traceability` is a pure projection over selected Scenarios with inherited parent realizations,
derived Check relationships, Challenge resolution accounts and decision-impact edges. It creates
no authored authority or execution fact.

`azimuth export` writes the complete derived model as format version 8. The export includes Requirements,
Scenarios, workspace data, realization and implementation linkage, mechanisms, Checks, Evidence
Bindings, Verification Challenges, Challenge resolutions and Findings. Historical declaration collections remain separately represented where that lane is present; runtime review histories are not repository export content.

The core reads language-neutral manifests rather than source. Ecosystem extractors emit the shared strict linkage collections. This keeps source parsing outside the core and makes a language integration an extractor concern instead of a fork of the model.

Nested change and project commands retain their bounded lifecycle meanings. They do not perform model validation or Check execution.

## Multi-repository assembly

A project may be assembled from independent repositories without making paths global identity. The project catalog declares required repositories, stable areas and model-source authorities. A workset supplies concrete revisions and content digests for repository manifests and execution receipts.

Every federated source has semantic identity `(area, typed address)`. Repository, mount and path are locators. Moving an unchanged area between repositories preserves linkage; splitting or merging an area is an explicit identity transition. Model-source authority follows intent, so code in one repository may realize a Requirement owned by another without copying the Requirement.

A local project result may be useful without being complete. Missing required inputs prevent a complete assembly and cannot produce finalization. Project acceptance compares complete pre- and post-archive accounts and verifies that one completed change moved unchanged within its singular authority.

## Changes and archive

Current facets describe accepted state. A **change** proposes a target state through intent deltas, solution design, implementation work and verification obligations. Proposed facts do not become current simply because they appear in a change directory.

Completion distils mechanisms that now exist and accepted intent into the current packages before archiving the whole change. The archive retains alternatives, departures and the semantic history of the transition. A change is not a branch, release or rollout; several repositories may contribute work while one repository retains change authority.

Criticality changes through the same lifecycle without changing Requirement identity. Raising a Requirement activates the applicable mechanism and evidence obligations. Lowering it records why those obligations no longer apply and what would raise it again.

## Merge-first lifecycle, observed context and reuse

A change is applied to the model and archived when it merges to main, before any release or deployment; main's model is the current model. A release carries no baseline, composes no deltas and is never accepted, and a deployment's Requirement Status is measured against its own release's model only. What main contains that an environment does not run is a separate delivery-gap report. Ratification of a model change is recorded in git (committed accepted reviews and, for non-routine Requirements, the owner's acceptance record) and verified by trusted CI, which publishes main's model as ratified or unratified; the Azimuth Assurance Service keeps no acceptance record. See [release-publication.md](../contracts/release-publication.md), [validation.md](../contracts/validation.md) and [run-ledger.md](../contracts/run-ledger.md).

A feature flag is not a new concept: it is a leaf of a component's observed Execution Context, which Azimuth reads and never sets. A Requirement or Scenario may declare a `Context:` condition naming a component and leaf, and a deployment whose captured context does not match shows it as not applicable here, which is not a violation. A Check lists the leaves its result depends on, and its evidence applies to a deployment only while those leaves hold the values the Run pinned. See [spec.md](../contracts/spec.md) and [deployment-requirement-status.md](../contracts/deployment-requirement-status.md).

Verification reuse is project policy, not per-Check authoring. One reviewed partition rule derives units; a unit's inputs are everything except other units' private paths, and a result of a unit declared hermetic may be reused for a release whose input fingerprint for that unit is equal. End-to-end Checks never take part. See [unit-inputs.md](../contracts/unit-inputs.md).

## Run and adapter execution control plane

Azimuth implements the provider-neutral [`azimuth-run-bundle` version 2](../contracts/run-bundle.md) exchange. One immutable bundle revision accounts for one logical Run over one exact typed Verification Target and semantic plan. It freezes planned and actual selection, physical activities, ordered attempts, one terminal Check Result for each actually selected Check and Scenario and one terminal Challenge Result for each selected Verification Challenge target. Current execution uses source snapshots, artifacts and deployments with content or deployed-state fingerprints rather than mutable locators. Local and CI origin belongs to provenance. Historical workspace, CI-candidate, service and monitoring-window descriptors remain parseable with their original meanings; they do not grant new cross-target reuse.

Run bundles reduce retries and finite work units deterministically. Violations and challenge findings survive retries; missing planned units and unfinished work cannot become positive results. An omitted target creates no fabricated result. Full replacement corrections retain immutable Verification Target, Execution Context, plan, source-execution and start anchors and form one fingerprint-linked chain.

The service-free protocol commands are:

```text
azimuth run verify --bundle <file> [--bundle <file> ...]
azimuth run inspect --bundle <file> [--bundle <file> ...] [--format text|json] [--out <file>]
```

Verification proves strict shape, internal identity, plan/actual agreement, reduction, references and correction history. A protocol-consistent violation, challenge finding or partial Run is an execution fact and therefore exits zero. Inspection presents a deterministic account and labels current repository authority and Assurance State unresolved. Neither command calls a provider, reads an artifact locator, writes to a service or treats protocol validity as product acceptance.

An explicit short-lived adapter boundary sits beside it. Strict `azimuth/adapters.json` configuration pins protocol and provider identity, executable and resource content, the description handshake, exact non-secret settings and environment literals, process limits and a capability dictionary. Core neither searches `PATH` nor invokes a shell. Its closed semantic capability classes are in [`contracts/adapter.md`](../contracts/adapter.md); provider families, configured `<adapter-id>/<capability-id>` addresses and Challenge forms remain open identities.

The reusable semantic Plan contains no provider route. A separate strict [`azimuth-run-launch-plan`](../contracts/run-launch-plan.md) version 2 binds the exact Verification Target, planned time, `execute | import` operation and complete semantic Plan to one configured adapter and one capability route for every selection. Route or configuration substitution changes the launch fingerprint and therefore the derived Run id.

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

Verification compares the running adapter's complete description with configuration. Planning loads the complete unselected model and accepts strict Check-only, Challenge-only or mixed requests; the two arrays are required and their union is non-empty. A Check request names its exact configured capability and finite units. A compact Challenge request selects a declared Verification Challenge, explicit capability, finite units and a bounded target selection. Core resolves assessment fingerprints and semantic scope from the account; historical Plan requests retain their separate native rules. There is no partial-model or `--only` mode.

Compact Challenge planning resolves declarations against current Requirement Review or Requirement Status fingerprints and produces bounded semantic scope and accountable launch inputs. Adverse or ambiguous target selection fails rather than silently choosing a larger source tree or whole suite. Historical authored Challenge Plans retain their own policy and scheduling rules; those rules are not extra mandatory compact review stages.

Each selection freezes an exact target and semantic scope. Source-backed scope items project into accountable source, Artifact, enumeration or surface-member locators. Semantic scope changes Plan identity; locator projection changes launch identity. An adapter translates this frozen account rather than reinterpreting repository authority.

Execute and import stage the configured executable, resources and import inputs from the same open streams core hashes. Core clears the child environment and invokes the staged executable directly. On a supported host, process creation places it in a fresh process group before adapter code runs. One configured deadline bounds request writing, concurrent response and diagnostic reads and core's own wait. Core signals remaining group members on every terminal path and cleans their inherited pipes while they retain group membership. A host without that process-group primitive rejects the exchange before spawn as an exit-one transport failure.

This boundary is not non-escapable descendant containment. Authorized adapter code can call `setsid`, `setpgid` or an equivalent and leave the group. An escaped descendant cannot extend core's wait beyond the deadline, but core does not guarantee its termination. The stage and host controls provide neither daemon supervision nor hostile-code isolation and are not a filesystem or network sandbox.

The adapter returns one strict response and complete bundle. Core validates description, request, launch, routes, provenance, actual selection, reduction and bundle identity before atomic output. Repeatable predecessors must be one verified correction chain; the adapter receives its complete terminal account and must return revision zero or exactly the next revision correcting that terminal fingerprint. A valid violated Check Result, Challenge finding, partial or cancelled Run, or adapter-returned protocol-valid `timed-out` Run fact is honest and exits zero. A host-enforced process deadline is a transport timeout and exits one, as does a semantic, identity, content or other transport mismatch. A `timed-out` Run is protocol-valid only when the adapter returns that complete fact within the host deadline; a host deadline produces no bundle. CLI and schema failure exits two. Neither nonzero class publishes an output.

Adapter, configuration, description, launch, capability routes, planned time and the complete normalizer join the correction anchors. Import-input identities remain protected in each revision but may change when later bytes from the same native execution arrive through the frozen route. A different adapter, capability or configuration therefore starts a different Run.

The current Run bundle version 2 requires this adapter provenance. No compatibility reader accepts an earlier version.

Generated plans currently represent Check and Challenge routes with model authority. Challenge Results are exactly `clean | findings | inconclusive`. Clean means only that the configured search found no objection; it creates no evidence or credibility. Every planned Challenge omitted from a partial, cancelled or timed-out Run has exactly one execution diagnostic scoped to its selection id and no fabricated Result. Scheduled omission is allowed deferral; gate omission records an honest execution failure. Added or substituted target, context, scope or units is a mismatch.

`model.extract` execution remains absent. The current Run ledger supplies authenticated durable ingestion, immutable histories and compact exact-Verification-Target Assurance State. Current planning defines no cache validity, cadence, historical-applicability or cross-target reuse semantics. Adapters remain bounded short-lived processes; there is no daemon, webhook, event gateway or long-running adapter boundary.

The synthetic [Challenge-planning conformance](../experiments/challenge-planning/README.md) uses only
the public commands and exercises all twelve selector forms, mixed planning, exact scope and routes,
scheduled omission, import provenance and selection mismatch without creating persistent state.

The earlier Assurance Service remains isolated beside the current Run ledger. Its earlier service API and version 2 project-snapshot wire remain isolated inside that separate service boundary. A snapshot Case is opaque `caseStatement` text, but the protocol is not the alpha 5 repository model or Run-bundle protocol, is not emitted by `azimuth export`, and receives no compatibility bridge. The current ledger supplies authorization, durable ingest, immutable history and compact Verification-Target-specific Assurance State.

The authority split is current: repositories own Requirements, Checks and binding meaning; Run producers own execution facts about exact Verification Targets, and independently authenticated reviewers own compact review records. The older declaration lane retains its repository-owned decisions. A standalone valid bundle does not establish that its model or decision fingerprints are current.

Historical consumer feedback is retained only as an [immutable provenance citation][historical-consumer-provenance]. The citation is documentary; no build, test, release or acceptance step reads that repository.

[historical-consumer-provenance]: https://github.com/drim-dev/azimuth-demo/blob/68a2eb5d46daf01ba087ec94b6a1ea7901c63bfd/azimuth/model/trips/rider-view/verification.md

## What is not claimed

One falsifier stays open and is cheap to run once the export carries non-routine content: if the intent, mechanism and evidence views over the export turn out to be substantially the same view, the three-facet split is decorative and the artifacts should collapse into fewer.

Azimuth does not prove prose predicates, infer honest linkage from source, turn a clean Verification Challenge search into positive product evidence, create a repository decision from execution facts or enroll native tests automatically. Its current outputs are a versioned repository account, derived traceability and validated bounded adapter exchanges. The current ledger adds durable ingestion and independently reviewed Verification-Target-specific assurance. Protocol validity alone still supplies no positive Requirement conclusion.

## Local work and accountable publication

Local draft reviews, debug Runs and correction attempts remain local by default. CI validates exact dependencies and publishes the current package relied on for delivery, including necessary provenance and relevant adverse or unresolved facts. All model-assisted review can happen locally; deterministic CI transport does not relabel an agent or human reviewer. Review submission requires the authorized reviewer credential matching the original record; a producer token authorizes Runs only.

The Assurance State Service preserves immutable published assessments, Run corrections and main-model publications. Definition review is visible independently of Verification Targets; a Requirement Status record is exact-target and cannot be reused merely because an image or model revision matches. Source-to-artifact-to-deployment provenance is explicit information, not automatic evidence applicability. Candidate registration, publication and execution never select the default model or authorize deployment implicitly.
