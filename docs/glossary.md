# Glossary

Bounded definitions for the current framework. Borrowed terms are narrowed where Azimuth cannot support their broader formal meaning. If another active document uses one of these terms differently, that document is wrong.

Modules organize Requirements. Scenarios describe expected behavior. Checks produce Results. The Azimuth Assurance Service (AAS) assesses the evidence and shows Requirement Status for a deployment.

---

## Requirement model

**Module** — a named, hierarchical area of the specification. A Module owns Requirements and can
contain nested Modules (for example `payments`, `payments/reminders`,
`payments/reminders/scheduling`). Parent views can aggregate Requirements from descendants;
hierarchy is declared explicitly rather than inferred from paths. A Module does not imply a code
package, service, repository or deployment boundary.

**Spec** — the authored intent document (`spec.md`) of one Module. Its declared id is
path-independent. Specs are organized by problem domain rather than by service.

**Requirement** — behavior or a constraint the system must satisfy: the independently governable
proposition about a product or its operation. Requirements belong in specifications; designs
explain how implementations meet them. A Requirement can express business, functional, security,
reliability or other system constraints. It owns criticality, production realization, total
Requirement Judgment and Assurance State.

**Scenario** — a concrete situation and its expected outcome within one Requirement, commonly
expressed with Given/When/Then or the authored `Context` / `Event` / `Required outcome` lines. A
Scenario describes behavior; it is distinct from the Check that verifies it. It is addressable for
Evidence Bindings, Runs, Check Results and Verification Challenge impact, but owns no independent
criticality, realization, Judgment or Assurance State. Its stable project-wide ID is independent of
its current Module and parent Requirement.

**Predicate** — what must hold, written as free-form normative Markdown in a Requirement and its
Scenarios. *Narrowing:* core does not interpret its natural language, tables, diagrams, code fences
or domain notation. Azimuth preserves and fingerprints the authored content and checks the
structure of its assurance account, not the truth of the content.

**Domain** — what a Requirement ranges over. The parser assigns it structurally: a Requirement with
authored Scenarios takes `behaviour`; an `Invariant` with an `Over:` surface takes `sites` and
receives one implicit Scenario.

**Quantifier** — deliberately absent from a Requirement. Requirements are universal; a constant field would carry no information. Evidence Binding quantification describes the breadth of a Check, not the logical quantifier of the Requirement.

**Criticality** — how much a Requirement matters, declared on every Requirement and changeable
without changing its identity. Scenarios inherit it but do not own it.

## Facets and authority

**Facet** — one of the three accountabilities for a Requirement: intent, mechanism and evidence. Which facets are applicable depends on criticality.

**Intent** — what must be true, over what domain and how much it matters. Recorded in `spec.md`.

**Mechanism** — what makes a Requirement true. Recorded in `design.md` when applicable.

**Evidence facet** — the repository-owned account of which deliberately enrolled methods bear on
which Scenarios and why. Compact `verification.md` owns Checks and inline Evidence Bindings; independently authored Method Qualifications, exact-target Applicability Decisions and Requirement Judgments are separate strict review records. The older declaration lane records those decisions in `verification.md`.

**Requirement assurance record** — the logical body of assurance information associated with one
Requirement. Its definition-time part is the per-Requirement cluster of authored Module documents
(`spec.md`, `design.md`, `verification.md` and `judgments.md` where present) assembled into one
account. AAS separately holds the Requirement Reviews, Runs, Check Results, Execution Context
captures and derived assessments that bear on it. *Narrowing:* it names that collection only; there
is no first-class aggregate, table, API resource or lifecycle with this name. Broader scopes such as
the installation account remain distinct.

**Residue** — knowledge that belongs to no Requirement, such as orientation, danger zones and deliberately absent behavior. It is outside the model and creates no assurance relation.

**Facet owner** — whoever is accountable for the sufficiency of one facet. Intent owner, mechanism owner and evidence owner describe accountabilities, not required job titles or exclusive authorship rights.

**Repository authority** — durable model meaning owned and reviewed with source: Requirements,
Scenarios, criticality, mechanisms, Checks and authored Evidence Bindings. The older declaration lane also owns Method Qualifications, Applicability Decisions, Verification Challenges, Challenge Plans, Requirement Judgments, Decision Policies and the Challenge Schedule in repository documents. Compact reviews remain separate accountable records over exact repository definitions.

## Verification graph

**Check** — a deliberately enrolled verification method with one atomic terminal proposition: a
defined way to verify one or more Scenarios through explicit Evidence Bindings. A Check can execute
before deployment or against a deployed system; its declared execution scope and deployment
applicability decide which.

**Monitor** — a recurring production Check, declared by the Check's deployment metadata. It is not a
separate verification primitive.

**Check implementation** — one compiler- or extractor-resolved source site linked to a Check by `ImplementsCheck(<check-id>)`. It supplies implementation identity and an exact source fingerprint, but declares no Requirement relation or evidence form. Several sites may compose one Check.

**Evidence Binding** — one authored relation from a Check's proposition to one Scenario. The unique typed `(Check, Scenario)` pair identifies a compact binding; its inline Contribution states the narrower evidentiary meaning. It has no separate authored binding ID, Execution Context, Decision Policy or Method Qualification field. The older declaration lane additionally records those fields.

The following form/context/policy fields belong to the older declaration lane; they are not implicit compact binding fields.

**Scope** — defined by what must be real for the Check rather than by how many processes happen to execute. The closed rungs are in [`contracts/verification.md`](../contracts/verification.md).

**Quantification** — whether a Check evaluates one case or ranges over a derived or generated set. *Narrowing:* the broad value describes the method's declared breadth, not mathematical exhaustiveness. The closed values are in [`contracts/verification.md`](../contracts/verification.md).

**Oracle** — the source of the expected result for an Evidence Binding. The vocabulary is descriptive and is not a ranking; its closed values are in [`contracts/verification.md`](../contracts/verification.md). Three values are routinely confused and are distinguished by what the expectation is drawn from: `relational` states a relation among values observed for **one** case; `metamorphic` states a relation **across executions** connected by an intentional transformation; `model-based` computes the exact expected result from an **independent** model. One case is not one function or process — a request, its response, resulting persisted state and an outbound call gathered for the same case are one case.

**Required Execution Context** — the exact string-to-string Execution Context in which an Evidence Binding is qualified. Alpha 2 provides no wildcards, ranges or provider expressions.

**Challenge domain** — the relations a Verification Challenge may traverse from an Evidence Binding. It is not a list of provider products; the closed set is in [`contracts/verification.md`](../contracts/verification.md).

**Method Qualification** — an independent review of the credibility of an exact Check method. Compact records pin the Check fingerprint and use `accepted | rejected`; shared bindings may rely on the same current qualification. The older declaration lane uses `qualified | rejected` and pins its declared form, oracle, Execution Context, challenge domain and policy.

**Applicability Decision** — the independent review that qualified evidence bears on one Check–Scenario contribution. Compact records use `accepted | rejected`, pin the exact Verification Target, current binding and qualification, and reference exact Run/bundle/Check Result basis. The older declaration lane uses `applicable | rejected` and the authored binding ID.

**Decision Policy** — a project standard naming open Verification Challenge forms required for a
Method Qualification, Applicability Decision or Requirement Judgment.

**Challenge Schedule** — the one project account assigning every required or declared Challenge
form exactly once to `gate | scheduled`. The lane never changes repository decision identity.

**Verification Challenge** — an activity that tests whether verification detects defects or
incorrect behavior: a method that searches for a reason to distrust an exact assessment. Current
compact targets are Requirement Review records; the older declaration lane
targets a Method Qualification, Applicability Decision or Requirement Judgment.

**Challenge Plan** — a repository declaration pairing one Verification Challenge with twelve
semantic selector forms for exact current Method Qualification, Applicability Decision and
Requirement Judgment fingerprints.

**Challenge candidate disposition** — how one resolved candidate stands. Only a current positive decision is selected, and adverse siblings remain visible. The closed dispositions are in [`contracts/verification.md`](../contracts/verification.md).

**Requirement Judgment** — the independent assessment of a parent Requirement's complete Scenario, mechanism and applicable-evidence composition. Compact records pin an exact Verification Target and complete current dependencies, with `supported | violated | unresolved` conclusions and residual risks. The older declaration lane uses a repository-owned `accepted | rejected` decision.

**Decision impact edge** — the pure projection from a challenged decision through its exact
dependants to the parent Requirement and current Requirement Judgment. Method impact fans out;
applicability impact stays local. It never creates another Challenge Result.

**Ordinary engineering test** — an unenrolled native test used to build confidence in the implementation. It creates no Azimuth evidence relation and needs no exemption. All tests for the current routine Requirements are in this category.

## Verification packages

**Verification package** — shared authoring concepts, source contracts and validation rules that extend the core Check–Scenario account. Packages may compose; they are not alternative Requirement types or competing evidence authorities.

**Surface** — a declared finite inventory domain with published field names. A package-owned enumerator derives actual members and their language-independent descriptor from source. This verification Surface is distinct from the older workspace surface/domain declaration.

**Expectation Set** — independently authored expected records over one Surface. Several Sets may support different Checks over the same enumeration. Equality and coverage Checks do not generate the expected policy from the metadata they inspect.

**Probe** — a package-owned measurement implementation used by Checks for declared origins. It supplies measurement provenance and health, not automatic Scenario meaning or a Requirement verdict.

**Authority projection** — the full unselected model's pinned Checks, Scenarios, bindings, Requirements, mechanisms and structural gaps. The ledger owner explicitly selects it; exporting or registering a candidate does not accept a change.

## Mechanism and linkage

**Mechanism identity** — a stable, design-owned id for one atomic enforcement mechanism. It is independent of a code symbol, so deleting an implementation leaves an unresolved declaration instead of erasing both sides of the relation.

**Enforcement kind** — how a mechanism prevents a violation. Kinds are mechanism properties, not executable Check forms; the closed ladder is in [`contracts/design.md`](../contracts/design.md).

**Choke point** — a single place through which a violation would have to pass. Contrast with a guard repeated at every site.

**Tag** — a machine-readable source annotation. Current tags express realization, mechanism implementation or Check implementation only. They do not assign evidence meaning.

**`realizes`** — a production relation saying that the source site establishes some part of a
parent Requirement's predicate. It names the project-global Requirement ID and carries no Scenario
or evidence form.

**`implements-mechanism`** — a production relation binding a compiler-resolved symbol to one declared design mechanism. Its source annotation names the project-global Mechanism ID; the extractor derives qualified identity.

**Mechanism implementation site** — the ecosystem-semantic qualified `site` emitted with one marker-derived mechanism implementation. .NET uses namespace/type/method/metadata signature; JVM uses binary class/method/descriptor; TypeScript and JavaScript use package/module/receiver/symbol and canonical overloads; Go uses import path/receiver/function/typed signature with positional generics; Python uses the one root-relative module and `__qualname__`; Rust uses a conventional Cargo target, reachable module and normalized declared signature whose type-path spelling remains semantic; C++ accepts only an external-linkage, non-module, non-template, unconstrained program-global declaration and uses its qualified name and canonical function type. Ambiguity fails rather than consulting a file.

**Marker companion** — the exact Artifact paired with one raw mechanism implementation by `(id, kind, file)`. The raw implementation has exactly six fields and binding `<address-kind>:<site>`; companion id equals that binding. Assembly atomically rewrites both ids to `<area>|<address-kind>|<site>`. It is marker-only and cannot also satisfy an explicit Design binding.

**`ImplementsCheck`** — source linkage from one resolved implementation site to one project-global Check id. The Evidence Binding remains repository-owned.

**Design binding** — an explicit machine-addressable ordinary Artifact for a design mechanism. Without one, each tagged source site contributes its own derived binding and companion Artifact; several distinct sites may support one mechanism.

**Delivery topology** — the exchange, bindings, queues and failure routes that connect a brokered producer to consumers. It is a realization site when correct routing is part of the Requirement.

**Fan-out** — one Requirement realized at several sites across components or languages.

**Exemption** — a deliberate, attributable and reviewable opt-out from an applicable obligation. *No exemption record, block or Finding kind exists in alpha 5*; the term is reserved. An ordinary unenrolled test asserts no Azimuth evidence and therefore has nothing to exempt.

## Areas and derived domains

**Area** — a stable source namespace owned by one repository in a project snapshot. Its mounts locate code, tests, migrations or non-code artifacts. It is not a spec and does not imply evidence scope.

**Typed source address** — an extractor-defined address such as a symbol, export, route or index. `(area, address kind, address)` is stable source identity; repository, mount and path are locators.

**Enumerator** — a mechanism that derives all members of a site domain from the same source used to build the system. A hand-maintained list is not an independent witness.

**Surface** — a named site domain assembled from independently derived enumerator contributions. It answers which sites participate in a universal domain, not which business Requirements they realize.

**Realization obligation** — an optional requirement that an applicable Requirement have at least one realization in each named area. It constrains architectural participation and creates no evidence edge.

## Validation and outputs

**Finding** — one deterministic validation result with a stable kind, closed category, severity, source location, optional Requirement and criticality, detail and corrective help. The exhaustive registry and the closed categories are in [`contracts/findings.md`](../contracts/findings.md).

**Validation** — deterministic interpretation of the derived repository model. `azimuth validate` reports Findings without executing Checks.

**Traceability report** — a pure derived view of selected Scenarios, inherited parent
realizations, Check relationships, Challenge resolutions and decision-impact edges.

**Export** — the complete derived repository model serialized as format version 8 by `azimuth
export`. It includes Requirements, Scenarios, decisions, Challenge resolutions and Findings, but no
runtime ledger data.

**Machine tier** — deterministic model validation. It reports structural Findings and cannot establish truth.

**Agent tier** — accountable semantic review of relations and propositions the machine cannot interpret, such as whether a Check is discriminating or a realization earns its tag. *The model records no agent-tier artifact.* Review happens outside the tool and reaches the repository only as an authored `Qualifier:` or `Judge:` identity on a decision, which is accountable for the verdict however it was drafted.

## Project and change process

**Project** — the complete Azimuth repository account assembled from declared model sources and repositories. Completeness is declared by a project catalog rather than inferred from whichever inputs happen to be present.

**Repository** — an independent version-control and delivery boundary. It owns areas and may own model sources, but a Requirement or change need not be confined to one repository.

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

**Verification Target** — the exact thing being verified: the source snapshot, build artifact or
deployment about which a Run asserts execution facts. The [`azimuth-run-bundle`
format](../contracts/run-bundle.md) defines the closed Verification Target variants; historical
workspace, CI-candidate, service and monitoring-window variants keep their original meanings.
Provider locators remain provenance rather than exact state. *Narrowing:* unrelated subjects, such
as an OIDC subject or a release attestation subject, are not Verification Targets.

**Execution Context** — the behavior-relevant conditions under which a Check or an application
instance executes. A **Run Execution Context** records the relevant conditions under which a Check
executed; an **Application Execution Context** records the relevant conditions under which an
application instance operates. The application reports facts; AAS determines whether those facts
establish evidence applicability.

**Context Contract** — an owner-registered definition of the relevant Execution Context properties
and their comparison rules, identified by its canonical fingerprint. It declares which nonsecret
fields may be reported; it does not configure the application.

**Context endpoint** — the application endpoint (`/internal/execution-context` by default) that
reports the application's effective Execution Context as an `azimuth-execution-context` version 2
report for the Collector.

**Run** — one bounded, provider-neutral logical execution of selected Checks or Verification
Challenges over one exact Verification Target and semantic plan, including its Execution Context,
timing and completion state. Native processes, parameters, shards and retries are subordinate
details. The Run envelope is distinct from the Check executions and Check Results it contains. A
Check executes inside a Run and never emits one.

**Semantic Plan** — the provider-neutral selection inside a Run bundle. It freezes complete-model identity, exact required Execution Context, Check implementations, Verification Challenge targets, schedule lanes, semantic scope and finite work units without naming an adapter or native selector.

**Semantic Challenge scope** — the sorted, unique provider-neutral account of selector anchors and complete decision inputs for one Challenge selection. Every item has a closed kind, stable id and canonical fingerprint. Overlapping selectors union scope; conflicting fingerprints fail.

**Accountable launch input** — the exact route-local projection of one source-backed semantic scope item. It repeats kind, id and fingerprint plus a strict source, Artifact, enumeration or surface-member locator account. Locators affect launch identity, never semantic Plan identity.

**Run launch plan** — the strict provider-routing account that binds one exact Verification Target, planned time, `execute | import` operation and complete semantic Plan to one configured adapter and one capability route per selection. Changing a route changes launch identity and the derived Run id.

**Run bundle** — one strict immutable JSON revision accounting for a Run's Verification Target, Execution Context, planned and actual semantic selection, activities, attempts, terminal results and provenance. Versioned canonical fingerprints identify its semantic components and complete content.

**Check Result** — the outcome recorded by a Check execution: the Scenario-addressed terminal result
for one actually selected Check in one Run, with outcome `satisfied | violated | inconclusive`. One
physical Check execution produces exactly one Check Result per selected Scenario. A satisfied Check
Result alone does not establish a satisfied Requirement.

**Observation** — an observed runtime fact, such as actual deployed composition, effective
application configuration, runtime instance identity, or a fact collected before its relationship
to a Check Result has been established. *Narrowing:* a raw target-observation envelope remains an
Observation even when its normalized payload produces a Run and Check Result.

**Evidence** — information used to assess a Requirement, such as Check Results, Execution Context
captures and Observations whose applicability has been established. *Narrowing:* Evidence is an
umbrella concept with no record of its own; information whose applicability is not established is
not Evidence for that Requirement.

**Challenge Result** — the terminal result for one selected Verification Challenge and exact
assessment fingerprint. A clean result records a negative search, not positive product evidence.

**Scheduled omission** — absence of one planned `scheduled` Challenge from an incomplete Run. It has exactly one execution diagnostic scoped to the Challenge-selection id and no Challenge Result. It is not a fourth result and does not make the selection disappear from the Plan.

**Correction** — the complete next immutable revision of one Run bundle. It names its immediate predecessor and preserves Run identity, Verification Target, plan, required and actual Execution Context, source system and execution, started time, adapter identity and version, descriptor, configuration, launch, capability routes, complete normalizer and planned time. Import-input identities are protected per revision but may change when later bytes from the same native execution arrive through the frozen route. A correction is not a patch or timestamp-selected winner.

**Protocol-valid** — internally consistent under the Run-bundle schema, identities, selection, reduction, references and correction rules. It says neither that repository fingerprints are current nor that Assurance State is acceptable.

**Assurance State** — a dynamic conclusion for one exact Verification Target, derived from repository decisions and accepted execution facts.

**Adapter** — an explicitly configured, short-lived provider process that translates core-selected semantic targets to native work or imports an exact native report. It returns a provider-neutral Run bundle and never interprets repository meaning independently. Core invokes it without a shell, `PATH` discovery or ambient environment inheritance.

**Adapter configuration** — the strict `azimuth/adapters.json` account that pins adapter and provider identity, executable and resource content, description, exact non-secret settings and environment literals, process limits and capabilities. Locators do not substitute for content identity.

**Capability class** — the semantic role a configured adapter capability fills. The closed classes are in [`contracts/adapter.md`](../contracts/adapter.md); `model.extract` is declared but has no current execution command.

**Configured capability address** — the open `<adapter-id>/<capability-id>` route identity for one configured capability. It is distinct from the open provider-family identity and from an open Challenge form.

**Bounded adapter exchange** — one strict request and response with same-stream content staging, a cleared child environment, independent output bounds and one deadline for core request writing, response and diagnostic reads and process wait. On supported hosts, the adapter starts in a fresh process group and core signals remaining group members on every terminal path. Authorized code may use `setsid`, `setpgid` or an equivalent to leave the group; it cannot extend core's wait beyond the deadline, but its termination is not guaranteed. This is not non-escapable descendant containment, daemon supervision, hostile-code isolation or a filesystem or network sandbox.

A protocol-valid adapter-returned `timed-out` Run is an execution fact and exits zero only when its complete response arrives within that deadline. A host-enforced deadline is a transport timeout, exits one and publishes no bundle.

**Azimuth Assurance Service (AAS)** — the current project-scoped `run-ledger` persists valid Run histories, main-model publications and independent compact reviews, then derives exact-Verification-Target state. It also holds the separate, non-assessment Work Provenance ledger. Its producer, reviewer and owner roles authorize different actions. The earlier service API and version 2 project-snapshot wire stay isolated with no compatibility bridge. A bounded adapter is not this ledger or a long-running webhook host.

## Current compact assessment vocabulary

**Requirement Review** — an independent assessment of whether the proposed verification is a
sensible way to verify the Requirement and whether that verification has been implemented
correctly. It examines exact Requirement/Scenario definitions, design, implementation, Check
contributions, oracles, scope and limitations, and records `accepted | rejected`. No execution
Verification Target or Check Results are required. It is distinct from deployment Requirement
Status.

**Requirement Status** — the conclusion established by applicable evidence about a Requirement for
a deployment. AAS computes it from reviewed definitions, release provenance, the observed deployment
and compatible Execution Context captures. Internal values `supported | violated | unresolved` are
presented as **Satisfied**, **Not satisfied** and **Unresolved**. Missing, stale, conflicting or
incomplete support remains unresolved. A passing Check or an accepted Requirement Review alone never
implies a satisfied Requirement. Requirement Status is always computed; there is no authored
Requirement Status record.

**Verification Challenge implementation** — tagged machine-executable code/configuration
implementing a declared search for objections to a Requirement Review. Production targets use ordinary Requirement/Mechanism implementation tags, not separate
Verification-Challenge-specific target tags.

Method Qualification, Applicability Decision and Requirement Judgment definitions above describe
historical records; they are not mandatory steps in the current compact assessment lane.

## Work Provenance vocabulary

**Work Provenance** — the ledger of completed workflow attempts, kept as a separate domain that is
never an input to Requirement Review currentness, Requirement Status or acceptance. It records
activities, not their products: a review, design or implementation lives in its own domain and is
referenced by fingerprint. See [`contracts/work-provenance.md`](../contracts/work-provenance.md).

**Attempt** — one completed, submitted workflow result: action, outcome, actor, agent descriptor,
times, guidance and the entities it used and generated. An attempt that never submitted a result is
harness telemetry, not provenance.

**Actor** — the execution instance that performed an attempt: one session, subagent run or person.
It is the reviewer-independence key and encodes no role, attempt number or model.

**Agent descriptor** — the closed description of what kind of agent performed an attempt: `kind`,
provider-canonical `model`, provider-native `effort` and `harness` name and version. Each value
carries its basis: `harness` (attested by the harness's own records), `declared` or `unknown`.

**Requirement work history** — the changes whose release altered a Requirement's closure, labelled
`intent` when the Requirement or its Scenarios changed and `dependency` otherwise, with their
attempts.

**Failure Attribution** — an optional, append-only record by which an investigator names the
attempts believed to have caused an adverse fact or allowed it through, with rationale and
confidence. It supports causal analysis and is never an assessment input.
