# Glossary

Bounded definitions for the current framework. Borrowed terms are narrowed where Azimuth cannot
support their broader formal meaning. If another active document uses one of these terms
differently, that document is wrong.

---

## Claim model

**Claim** — an addressable proposition about a product or its operation. A requirement-level
Claim states the normative proposition and owns criticality. A case-level Claim refines one
observable condition without becoming a different kind of model object. One case result does not
imply that it exhausts its requirement-level parent.

**Predicate** — what must hold, written in prose. *Narrowing:* it is not a formal predicate and
has no machine-evaluable semantics. Azimuth checks the structure of an assurance account, not the
truth of its prose.

**Spec** — a named group of requirements with a declared, path-independent id. Specs are organized
by problem domain rather than by service.

**Requirement** — a requirement-level Claim expressed as one normative SHALL proposition. It owns
criticality and groups case-level Claims.

**Scenario** — the GIVEN/WHEN/THEN syntax used to write a case-level Claim. Scenario ids are
unique per spec, not per requirement, so splitting or merging a requirement does not change case
identity.

**Domain** — what a Claim ranges over. The closed values are executions of a behavior, a set of
sites, the code artifact itself, paired derivations that must agree, aggregate state over time and
eventual absence (D13.3).

**Quantifier** — deliberately absent from a Claim. Claims are universal; a constant field would
carry no information. Evidence Binding quantification describes the breadth of a Check, not the
logical quantifier of the Claim.

**Criticality** — `critical | standard | routine`. It is declared on every requirement and may
change without changing Claim identity. Routine Claims stop at intent and owe no realization,
mechanism or evidence linkage. Standard and critical Claims activate the applicable additional
facets. Every active Claim in this repository is routine during the alpha 2 phase.

## Facets and authority

**Facet** — one of the three accountabilities for a Claim: intent, mechanism and evidence. Which
facets are applicable depends on criticality.

**Intent** — what must be true, over what domain and how much it matters. Recorded in `spec.md`.

**Mechanism** — what makes a Claim true. Recorded in `design.md` when applicable.

**Evidence facet** — the repository-owned account of which deliberately enrolled methods bear on
which case-level Claims and why they are credible. Recorded through Checks, Evidence Bindings and
Qualifications in `verification.md`. It contains no execution result.

**Residue** — knowledge that belongs to no Claim, such as orientation, danger zones and
deliberately absent behavior. It is outside the model and creates no assurance relation.

**Facet owner** — whoever is accountable for the sufficiency of one facet. Intent owner,
mechanism owner and evidence owner describe accountabilities, not required job titles or exclusive
authorship rights.

**Repository authority** — durable model meaning owned and reviewed with source: Claims,
criticality, mechanisms, Checks, Evidence Bindings, Qualifications, Challengers, Challenge Plans,
standards and rationale.

## Verification graph

**Check** — a deliberately enrolled verification method with one atomic terminal proposition. It
directly evaluates one or more product or operational case-level Claims through explicit Evidence
Bindings. A Check is not every native test, analyzer rule or monitor.

**Check implementation** — one compiler- or extractor-resolved source site linked to a Check by
`ImplementsCheck(<check-id>)`. It supplies implementation identity and an exact source fingerprint,
but declares no Claim relation or evidence form. Several sites may compose one Check.

**Evidence Binding** — one repository-owned relation from a Check's atomic terminal proposition to
one case-level Claim. It states the edge proposition, evidence form, exact required context,
challenge domain and Qualification policy. Each `(Check, Claim)` pair is unique.

**Scope** — `unit | component | e2e`, defined by what must be real for the Check rather than by
how many processes happen to execute.

**Quantification** — `example | universal`, describing whether a Check evaluates one case or
ranges over a derived or generated set. *Narrowing:* `universal` describes the method's declared
breadth, not mathematical exhaustiveness.

**Oracle** — the source of the expected result for an Evidence Binding:
`direct | golden | relational | metamorphic | model-based | contract`. The vocabulary is
descriptive and is not a ranking.

**Required context** — the exact string-to-string context in which an Evidence Binding is
qualified. Alpha 2 provides no wildcards, ranges or provider expressions.

**Challenge domain** — the closed set of relations a Challenger may traverse from an Evidence
Binding: `realization | mechanism | check-implementation | oracle | context`. It is not a list of
provider products.

**Qualification** — the reviewed decision about whether one exact Evidence Binding is credible in
its required context. Its id is the binding id and its verdict is `qualified | rejected`. It
neither records an execution nor establishes that the Claim is satisfied.

**Qualification fingerprint** — the versioned SHA-256 identity combining the Check, binding and
exact-context fingerprints. Semantic source and Claim changes stale it; paths, line numbers,
mounts, criticality and explanatory prose do not.

**Qualification policy** — a project standard naming the Challenger forms required for an
evidentiary class. Its content participates in dependent binding fingerprints. Declaring a policy
does not execute a Challenger.

**Challenger** — a method that searches for a reason to distrust a Qualification. The proposition,
not the executable brand, determines the role. Challengers are not recursively qualified in alpha
2.

**Challenge Plan** — a repository declaration that pairs one Challenger with semantic selectors
for exact current decision fingerprints. Current selectors can traverse bindings, Checks,
realizations and mechanisms to Qualifications. Resolution is sorted and deduplicated; zero matches
are a Finding, never an implicit whole-suite fallback.

**Claim Judgment** — the future reviewed decision about the total assurance composition for one
Claim. It is distinct from a binding-level Qualification. Alpha 2 reserves Challenge Plan selector
syntax for it but has no current authoring format or command.

**Ordinary engineering test** — an unenrolled native test used to build confidence in the
implementation. It creates no Azimuth evidence relation and needs no exemption. All tests for the
current routine Claims are in this category.

## Mechanism and linkage

**Mechanism identity** — a stable, design-owned id for one atomic enforcement mechanism. It is
independent of a code symbol, so deleting an implementation leaves an unresolved declaration
instead of erasing both sides of the relation.

**Enforcement kind** — one of `type | schema | constraint | choke-point | middleware | guard`.
The kinds distinguish how violations are prevented. They are mechanism properties, not executable
Check forms.

**Choke point** — a single place through which a violation would have to pass. Contrast with a
guard repeated at every site.

**Tag** — a machine-readable source annotation. Current tags express realization, mechanism
implementation or Check implementation only. They do not assign evidence meaning.

**`realizes`** — a production relation saying that the source site is on a case-level Claim's
realization path. It is keyed by `(spec-id, case-id)` and carries no evidence form.

**`implements-mechanism`** — a production relation binding a compiler-resolved symbol to one
declared design mechanism.

**`ImplementsCheck`** — source linkage from one resolved implementation site to one project-global
Check id. The Evidence Binding remains repository-owned.

**Design binding** — the single machine-addressable artifact resolved for a design mechanism. It
may be explicit for a non-code artifact or derived from one implementation tag.

**Delivery topology** — the exchange, bindings, queues and failure routes that connect a brokered
producer to consumers. It is a realization site when correct routing is part of the Claim.

**Fan-out** — one Claim realized at several sites across components or languages.

**Exemption** — a deliberate, attributable and reviewable opt-out from an applicable obligation.
An ordinary unenrolled test asserts no Azimuth evidence and therefore has nothing to exempt.

## Areas and derived domains

**Area** — a stable source namespace owned by one repository in a project snapshot. Its mounts
locate code, tests, migrations or non-code artifacts. It is not a spec and does not imply evidence
scope.

**Typed source address** — an extractor-defined address such as a symbol, export, route or index.
`(area, address kind, address)` is stable source identity; repository, mount and path are locators.

**Enumerator** — a mechanism that derives all members of a site domain from the same source used
to build the system. A hand-maintained list is not an independent witness.

**Surface** — a named site domain assembled from independently derived enumerator contributions.
It answers which sites participate in a universal domain, not which business Claims they realize.

**Realization obligation** — an optional requirement that an applicable Claim have at least one
realization in each named area. It constrains architectural participation and creates no evidence
edge.

## Validation and outputs

**Finding** — one deterministic validation result with a stable kind, closed category, severity,
source location, optional Claim and criticality, detail and corrective help. The exhaustive kind
registry drives detailed output and counts.

**Validation** — deterministic interpretation of the derived repository model. `azimuth validate`
reports Findings without executing Checks.

**Traceability report** — a pure derived view of selected case-level Claims, ordered realization
identities and Check relationships. `azimuth report traceability` creates no authored authority or
execution fact.

**Export** — the complete derived repository model serialized as format version 2 by
`azimuth export`. It includes Findings and verification graph declarations, but no runtime ledger
data. Alpha 2 has no assurance-specific export.

**Machine tier** — deterministic model validation. It reports structural Findings and cannot
establish truth.

**Agent tier** — accountable semantic review of relations and propositions that the machine
cannot interpret. For the implemented verification graph, it can propose a binding-level
Qualification for evidence-owner review. It does not turn its own review into product evidence.

## Project and change process

**Project** — the complete Azimuth repository account assembled from declared model sources and
repositories. Completeness is declared by a project catalog rather than inferred from whichever
inputs happen to be present.

**Repository** — an independent version-control and delivery boundary. It owns areas and may own
model sources, but a Claim or change need not be confined to one repository.

**Model source** — the authority owning a set of intent packages. Ownership follows intent
accountability rather than implementation placement.

**Project reference** — a repository-local locator for the singular project catalog and,
optionally, an integration workset. It is not a second topology authority.

**Workset** — concrete repository checkouts and pinned manifests used for one assembly. A workset
may be partial; its contents never redefine project completeness.

**Repository manifest** — a versioned, revision-bound record emitted by one repository. It names
areas, model-source digests, changes, producer and typed linkage records.

**Execution receipt** — a content-addressed result of composed engineering checks naming the exact
repository revisions evaluated. It is not a Run.

**Project snapshot** — a finalization record over a complete clean assembly: catalog digest, area
topology, revisions, manifest and receipt digests, and derived model fingerprint.

**Change** — the temporal envelope from accepted current state to a proposed target. It carries
intent deltas, design where needed, implementation work and verification obligations. It is not a
branch, merge request, artifact, environment or rollout.

**Change authority** — the one repository record containing a change id, active or archived.

**Archive** — the immutable semantic record of a completed, rejected or abandoned change.

**Finalization** — the derived model fingerprint and validation summary for an accepted applied
change. It gates the mechanical archive move and contains no authored risk decision.

**Rollout** — exposure of an accepted artifact across environments or user populations. It is
normally outside the change model.

## Run protocol and deferred runtime concepts

**Subject** — the exact workspace, CI candidate, artifact, deployment, service or bounded
monitoring window about which a Run asserts execution facts. The
[`azimuth-run-bundle` format](../azimuth/formats/run-bundle.md) defines the closed Subject variants;
provider locators remain provenance rather than exact state.

**Run** — one bounded, provider-neutral logical execution over one exact Subject and semantic
plan. Native processes, parameters, shards and retries are subordinate details. A Check executes
inside a Run and never emits one.

**Run bundle** — one strict immutable JSON revision accounting for a Run's Subject, context,
planned and actual semantic selection, activities, attempts, terminal results and provenance.
Versioned canonical fingerprints identify its semantic components and complete content.

**Observation** — the terminal `satisfied | violated | inconclusive` result for one actually
selected Check in one Run. It is an execution fact, not an Evidence Binding or Qualification, and
is not copied for every binding.

**Challenge Result** — the terminal `clean | findings | inconclusive` result for one selected
Challenger and exact Qualification or Claim Judgment target fingerprint. A clean result records a
negative search, not positive product evidence.

**Correction** — the complete next immutable revision of one Run bundle. It names its immediate
predecessor and preserves Run, Subject, context, plan, source-execution and start anchors. It is not
a patch or timestamp-selected winner.

**Protocol-valid** — internally consistent under the Run-bundle schema, identities, selection,
reduction, references and correction rules. It says neither that repository fingerprints are
current nor that Assurance State is acceptable.

**Assurance State** — a future dynamic conclusion for one exact Subject, derived from repository
decisions and accepted execution facts.

**Adapter** — a deferred provider-family integration that will translate core-selected semantic
targets to native selectors and produce Run bundles without interpreting repository meaning
independently. Plan generation, execution and native report import are not current Run commands.

**Assurance Service** — the optional future durable ledger for accepted Runs and derived Assurance
State. D42's version 1 service wire remains isolated until the Run-ledger replacement; it is neither
the alpha 2 repository-model format nor the Run-bundle protocol.
