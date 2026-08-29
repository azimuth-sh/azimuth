# Glossary

Bounded definitions for the current framework. Borrowed terms are narrowed where Azimuth cannot support their broader formal meaning. If another active document uses one of these terms differently, that document is wrong.

---

## Claim model

**Claim** — the independently governable proposition about a product or its operation. It owns
criticality, production realization, total Claim Judgment and Claim Assurance State.

**Case** — a normative condition/outcome clause within one Claim's predicate. It is addressable for
Evidence Bindings, Runs, Observations and Challenger impact, but owns no independent criticality,
realization, Judgment or Assurance State. Its exact id is `<spec>#<claim>/<case>`.

**Predicate** — what must hold, written in prose. *Narrowing:* it is not a formal predicate and has no machine-evaluable semantics. Azimuth checks the structure of an assurance account, not the truth of its prose.

**Spec** — a named group of Claims with a declared, path-independent id. Specs are organized by
problem domain rather than by service.

**Domain** — what a Claim ranges over. The parser assigns it structurally: a Claim with authored
Cases takes `behaviour`; an `Invariant` with an `Over:` surface takes `sites` and receives one
implicit Case.

**Quantifier** — deliberately absent from a Claim. Claims are universal; a constant field would carry no information. Evidence Binding quantification describes the breadth of a Check, not the logical quantifier of the Claim.

**Criticality** — how much a Claim matters, declared on every Claim and changeable without changing
its identity. Cases inherit it but do not own it.

## Facets and authority

**Facet** — one of the three accountabilities for a Claim: intent, mechanism and evidence. Which facets are applicable depends on criticality.

**Intent** — what must be true, over what domain and how much it matters. Recorded in `spec.md`.

**Mechanism** — what makes a Claim true. Recorded in `design.md` when applicable.

**Evidence facet** — the repository-owned account of which deliberately enrolled methods bear on
which Cases and why. Recorded through Checks, Evidence Bindings, Method Qualifications,
Applicability Decisions and total parent Claim Judgments in `verification.md`.

**Residue** — knowledge that belongs to no Claim, such as orientation, danger zones and deliberately absent behavior. It is outside the model and creates no assurance relation.

**Facet owner** — whoever is accountable for the sufficiency of one facet. Intent owner, mechanism owner and evidence owner describe accountabilities, not required job titles or exclusive authorship rights.

**Repository authority** — durable model meaning owned and reviewed with source: Claims, Cases,
criticality, mechanisms, Checks, Evidence Bindings, Method Qualifications, Applicability Decisions,
Challengers, Challenge Plans, Claim Judgments, Decision Policies and the Challenge Schedule.

## Verification graph

**Check** — a deliberately enrolled verification method with one atomic terminal proposition. It
directly evaluates one or more Cases through explicit Evidence Bindings.

**Check implementation** — one compiler- or extractor-resolved source site linked to a Check by `ImplementsCheck(<check-id>)`. It supplies implementation identity and an exact source fingerprint, but declares no Claim relation or evidence form. Several sites may compose one Check.

**Evidence Binding** — one repository-owned relation from a Check's atomic terminal proposition to
one Case. It states the edge proposition, exact edge context, challenge domain, policy and referenced
Method Qualification. Each `(Check, Case)` pair is unique.

**Scope** — defined by what must be real for the Check rather than by how many processes happen to execute. The closed rungs are in [`contracts/verification.md`](../contracts/verification.md).

**Quantification** — whether a Check evaluates one case or ranges over a derived or generated set. *Narrowing:* the broad value describes the method's declared breadth, not mathematical exhaustiveness. The closed values are in [`contracts/verification.md`](../contracts/verification.md).

**Oracle** — the source of the expected result for an Evidence Binding. The vocabulary is descriptive and is not a ranking; its closed values are in [`contracts/verification.md`](../contracts/verification.md). Three values are routinely confused and are distinguished by what the expectation is drawn from: `relational` states a relation among values observed for **one** case; `metamorphic` states a relation **across executions** connected by an intentional transformation; `model-based` computes the exact expected result from an **independent** model. One case is not one function or process — a request, its response, resulting persisted state and an outbound call gathered for the same case are one case.

**Required context** — the exact string-to-string context in which an Evidence Binding is qualified. Alpha 2 provides no wildcards, ranges or provider expressions.

**Challenge domain** — the relations a Challenger may traverse from an Evidence Binding. It is not a list of provider products; the closed set is in [`contracts/verification.md`](../contracts/verification.md).

**Method Qualification** — the `qualified | rejected` decision about exact shared Check-method
inputs: implementation, form, oracle, common context, challenge domain and policy.

**Applicability Decision** — the `applicable | rejected` decision that one qualified method bears
on one exact Evidence Binding and Case under edge context. Its id is the binding id.

**Decision Policy** — a project standard naming open Challenger forms required for a Method
Qualification, Applicability Decision or Claim Judgment.

**Challenge Schedule** — the one project account assigning every required or declared Challenge
form exactly once to `gate | scheduled`. The lane never changes repository decision identity.

**Challenger** — a method that searches for a reason to distrust a Method Qualification,
Applicability Decision or Claim Judgment.

**Challenge Plan** — a repository declaration pairing one Challenger with twelve semantic selector
forms for exact current Method Qualification, Applicability Decision and Claim Judgment
fingerprints.

**Challenge candidate disposition** — how one resolved candidate stands. Only a current positive decision is selected, and adverse siblings remain visible. The closed dispositions are in [`contracts/verification.md`](../contracts/verification.md).

**Claim Judgment** — the repository-owned `accepted | rejected` decision about one standard or
critical parent Claim's total composition, including every Case and evidence edge.

**Decision impact edge** — the pure projection from a challenged decision through its exact
dependants to the parent Claim and current Claim Judgment. Method impact fans out; applicability
impact stays local. It never creates another Challenge Result.

**Ordinary engineering test** — an unenrolled native test used to build confidence in the implementation. It creates no Azimuth evidence relation and needs no exemption. All tests for the current routine Claims are in this category.

## Mechanism and linkage

**Mechanism identity** — a stable, design-owned id for one atomic enforcement mechanism. It is independent of a code symbol, so deleting an implementation leaves an unresolved declaration instead of erasing both sides of the relation.

**Enforcement kind** — how a mechanism prevents a violation. Kinds are mechanism properties, not executable Check forms; the closed ladder is in [`contracts/design.md`](../contracts/design.md).

**Choke point** — a single place through which a violation would have to pass. Contrast with a guard repeated at every site.

**Tag** — a machine-readable source annotation. Current tags express realization, mechanism implementation or Check implementation only. They do not assign evidence meaning.

**`realizes`** — a production relation saying that the source site establishes some part of a parent
Claim's predicate. It is keyed by `(spec-id, claim-id)` and carries no Case or evidence form.

**`implements-mechanism`** — a production relation binding a compiler-resolved symbol to one declared design mechanism. Its source annotation retains exactly the spec and mechanism arguments; the extractor derives qualified identity.

**Mechanism implementation site** — the ecosystem-semantic qualified `site` emitted with one marker-derived mechanism implementation. .NET uses namespace/type/method/metadata signature; JVM uses binary class/method/descriptor; TypeScript and JavaScript use package/module/receiver/symbol and canonical overloads; Go uses import path/receiver/function/typed signature with positional generics; Python uses the one root-relative module and `__qualname__`; Rust uses a conventional Cargo target, reachable module and normalized declared signature whose type-path spelling remains semantic; C++ accepts only an external-linkage, non-module, non-template, unconstrained program-global declaration and uses its qualified name and canonical function type. Ambiguity fails rather than consulting a file.

**Marker companion** — the exact Artifact paired with one raw mechanism implementation by `(id, kind, file)`. The raw implementation has exactly seven fields and binding `<address-kind>:<site>`; companion id equals that binding. Assembly atomically rewrites both ids to `<area>|<address-kind>|<site>`. It is marker-only and cannot also satisfy an explicit Design binding.

**`ImplementsCheck`** — source linkage from one resolved implementation site to one project-global Check id. The Evidence Binding remains repository-owned.

**Design binding** — the single machine-addressable artifact resolved for a design mechanism. It may be explicit for a non-code artifact or derived with one implementation tag and its companion.

**Delivery topology** — the exchange, bindings, queues and failure routes that connect a brokered producer to consumers. It is a realization site when correct routing is part of the Claim.

**Fan-out** — one Claim realized at several sites across components or languages.

**Exemption** — a deliberate, attributable and reviewable opt-out from an applicable obligation. *No exemption record, block or Finding kind exists in alpha 4*; the term is reserved. An ordinary unenrolled test asserts no Azimuth evidence and therefore has nothing to exempt.

## Areas and derived domains

**Area** — a stable source namespace owned by one repository in a project snapshot. Its mounts locate code, tests, migrations or non-code artifacts. It is not a spec and does not imply evidence scope.

**Typed source address** — an extractor-defined address such as a symbol, export, route or index. `(area, address kind, address)` is stable source identity; repository, mount and path are locators.

**Enumerator** — a mechanism that derives all members of a site domain from the same source used to build the system. A hand-maintained list is not an independent witness.

**Surface** — a named site domain assembled from independently derived enumerator contributions. It answers which sites participate in a universal domain, not which business Claims they realize.

**Realization obligation** — an optional requirement that an applicable Claim have at least one realization in each named area. It constrains architectural participation and creates no evidence edge.

## Validation and outputs

**Finding** — one deterministic validation result with a stable kind, closed category, severity, source location, optional Claim and criticality, detail and corrective help. The exhaustive registry and the closed categories are in [`contracts/findings.md`](../contracts/findings.md).

**Validation** — deterministic interpretation of the derived repository model. `azimuth validate` reports Findings without executing Checks.

**Traceability report** — a pure derived view of selected Cases, inherited parent realizations,
Check relationships, Challenge resolutions and decision-impact edges.

**Export** — the complete derived repository model serialized as format version 3 by `azimuth
export`. It includes Claims, Cases, decisions, Challenge resolutions and Findings, but no runtime
ledger data.

**Machine tier** — deterministic model validation. It reports structural Findings and cannot establish truth.

**Agent tier** — accountable semantic review of relations and propositions the machine cannot interpret, such as whether a Check is discriminating or a realization earns its tag. *The model records no agent-tier artifact.* Review happens outside the tool and reaches the repository only as an authored `Qualifier:` or `Judge:` identity on a decision, which is accountable for the verdict however it was drafted.

## Project and change process

**Project** — the complete Azimuth repository account assembled from declared model sources and repositories. Completeness is declared by a project catalog rather than inferred from whichever inputs happen to be present.

**Repository** — an independent version-control and delivery boundary. It owns areas and may own model sources, but a Claim or change need not be confined to one repository.

**Model source** — the authority owning a set of intent packages. Ownership follows intent accountability rather than implementation placement.

**Project reference** — a repository-local locator for the singular project catalog and, optionally, an integration workset. It is not a second topology authority.

**Workset** — concrete repository checkouts and pinned manifests used for one assembly. A workset may be partial; its contents never redefine project completeness.

**Repository manifest** — a versioned, revision-bound record emitted by one repository. It names areas, model-source digests, changes, producer and typed linkage records.

**Execution receipt** — a content-addressed result of composed engineering checks naming the exact repository revisions evaluated. It is not a Run.

**Project snapshot** — a finalization record over a complete clean assembly: catalog digest, area topology, revisions, manifest and receipt digests, and derived model fingerprint.

**Change** — the temporal envelope from accepted current state to a proposed target. It carries intent deltas, design where needed, implementation work and verification obligations. It is not a branch, merge request, artifact, environment or rollout.

**Change authority** — the one repository record containing a change id, active or archived.

**Work package** — one dependency-ordered, path-isolated implementation slice inside a change. It names its objective, owned paths, dependencies and expected evidence so a worker can execute it without sharing mutable scope with another worker. A work package is not a change, branch or independent authority and its worker never finalizes or archives the parent change.

**Consumer resource cohort** — the release-owned set of bundled skills, templates, references and migration edges embedded in the CLI. It is independent of contributor skills used to develop Azimuth itself.

**Installation account** — the tracked `azimuth/installation.json` record of the running release cohort installed in one repository: selected agent integrations, adopted aliases and components, and every CLI-managed resource path and digest.

**Authoring reference** — the version-matched parser-sensitive artifact contract exposed by `azimuth reference`. It states current accepted and rejected structure; it does not own stage choreography or project-specific policy.

**Migration line** — the explicit release identity within which supported account-format migration edges accumulate. A different line is a no-migration compatibility boundary unless a release declares an edge across it.

**Archive** — the immutable semantic record of a completed, rejected or abandoned change.

**Finalization** — the derived model fingerprint and validation summary for an accepted applied change. It gates the mechanical archive move and contains no authored risk decision.

**Rollout** — exposure of an accepted artifact across environments or user populations. It is normally outside the change model.

## Run and runtime concepts

**Subject** — the exact workspace, CI candidate, artifact, deployment, service or bounded monitoring window about which a Run asserts execution facts. The [`azimuth-run-bundle` format](../contracts/run-bundle.md) defines the closed Subject variants; provider locators remain provenance rather than exact state.

**Run** — one bounded, provider-neutral logical execution over one exact Subject and semantic plan. Native processes, parameters, shards and retries are subordinate details. A Check executes inside a Run and never emits one.

**Semantic Plan** — the provider-neutral selection inside a Run bundle. It freezes complete-model identity, exact context, Check implementations, Challenger targets, schedule lanes, semantic scope and finite work units without naming an adapter or native selector.

**Semantic Challenge scope** — the sorted, unique provider-neutral account of selector anchors and complete decision inputs for one Challenge selection. Every item has a closed kind, stable id and canonical fingerprint. Overlapping selectors union scope; conflicting fingerprints fail.

**Accountable launch input** — the exact route-local projection of one source-backed semantic scope item. It repeats kind, id and fingerprint plus a strict source, Artifact, enumeration or surface-member locator account. Locators affect launch identity, never semantic Plan identity.

**Run launch plan** — the strict provider-routing account that binds one exact Subject, planned time, `execute | import` operation and complete semantic Plan to one configured adapter and one capability route per selection. Changing a route changes launch identity and the derived Run id.

**Run bundle** — one strict immutable JSON revision accounting for a Run's Subject, context, planned and actual semantic selection, activities, attempts, terminal results and provenance. Versioned canonical fingerprints identify its semantic components and complete content.

**Observation** — the Case-addressed terminal result for one actually selected Check in one Run.
One physical Check execution produces exactly one Observation per selected Case.

**Challenge Result** — the terminal result for one selected Challenger and exact Method
Qualification, Applicability Decision or Claim Judgment fingerprint. A clean result records a
negative search, not positive product evidence.

**Scheduled omission** — absence of one planned `scheduled` Challenge from an incomplete Run. It has exactly one execution diagnostic scoped to the Challenge-selection id and no Challenge Result. It is not a fourth result and does not make the selection disappear from the Plan.

**Correction** — the complete next immutable revision of one Run bundle. It names its immediate predecessor and preserves Run identity, Subject, plan, required and actual context, source system and execution, started time, adapter identity and version, descriptor, configuration, launch, capability routes, complete normalizer and planned time. Import-input identities are protected per revision but may change when later bytes from the same native execution arrive through the frozen route. A correction is not a patch or timestamp-selected winner.

**Protocol-valid** — internally consistent under the Run-bundle schema, identities, selection, reduction, references and correction rules. It says neither that repository fingerprints are current nor that Assurance State is acceptable.

**Assurance State** — a future dynamic conclusion for one exact Subject, derived from repository decisions and accepted execution facts.

**Adapter** — an explicitly configured, short-lived provider process that translates core-selected semantic targets to native work or imports an exact native report. It returns a provider-neutral Run bundle and never interprets repository meaning independently. Core invokes it without a shell, `PATH` discovery or ambient environment inheritance.

**Adapter configuration** — the strict `azimuth/adapters.json` account that pins adapter and provider identity, executable and resource content, description, exact non-secret settings and environment literals, process limits and capabilities. Locators do not substitute for content identity.

**Capability class** — the semantic role a configured adapter capability fills. The closed classes are in [`contracts/adapter.md`](../contracts/adapter.md); `model.extract` is declared but has no current execution command.

**Configured capability address** — the open `<adapter-id>/<capability-id>` route identity for one configured capability. It is distinct from the open provider-family identity and from an open Challenge form.

**Bounded adapter exchange** — one strict request and response with same-stream content staging, a cleared child environment, independent output bounds and one deadline for core request writing, response and diagnostic reads and process wait. On supported hosts, the adapter starts in a fresh process group and core signals remaining group members on every terminal path. Authorized code may use `setsid`, `setpgid` or an equivalent to leave the group; it cannot extend core's wait beyond the deadline, but its termination is not guaranteed. This is not non-escapable descendant containment, daemon supervision, hostile-code isolation or a filesystem or network sandbox.

A protocol-valid adapter-returned `timed-out` Run is an execution fact and exits zero only when its complete response arrives within that deadline. A host-enforced deadline is a transport timeout, exits one and publishes no bundle.

**Assurance Service** — the optional future durable ledger for accepted Runs and derived Assurance State. The alpha 1 service wire remains isolated until the Run-ledger replacement; it is neither the alpha 4 repository-model format nor the Run-bundle protocol. No current adapter is a long-running service or webhook bridge.
