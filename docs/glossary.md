# Glossary

Bounded definitions. Several terms here are borrowed from fields with established formal
meanings — proof, universal, quantification, evidence. **Where a term is borrowed, the entry
states how this framework's use is narrower than the original.** That narrowing is deliberate:
precise vocabulary is only an asset while the model behaves the way the vocabulary implies. A
term that writes a cheque the model cannot cash costs more credibility than it buys.

If a document uses one of these words in a different sense, the document is wrong.

---

## The claim model

**Claim** — an addressable proposition about the product or its operation, carrying a stable id.
A requirement-level Claim states the normative proposition and owns criticality. A case-level Claim
refines one observable condition without becoming a separate kind of model object. Evidence bears
on Claims; no single case-level result implies that it exhausts its requirement-level parent.

**Predicate** — what must hold. Written in prose. *Narrowing:* not a formal predicate. It has no
machine-checkable semantics; its truth is established by evidence, not by evaluation. This is the
single largest gap between the framework's vocabulary and formal methods, and it is why no
mechanism here ever claims to establish truth.

**Domain** — what a claim ranges over. Six values: executions of a behaviour, a set of sites, the
code artifact itself, paired derivations that must agree, aggregate state over time, eventual
absence. Closed set for now (D13.3).

**Quantifier** — deliberately absent. Every claim is universal; a constant field carries no
information (D13). The only existential statements are marginal capability claims.

**Spec** — a named group of requirements with a declared, path-independent id. Organized by
domain area, never by service.

**Requirement** — a requirement-level Claim expressed as one normative SHALL rule. It owns
criticality and groups its case-level Claims.

**Scenario** — a case-level Claim in GIVEN/WHEN/THEN form. It refines an observable condition and
remains independently addressable. Ids are unique per spec, not per requirement, so that splitting
or merging a requirement touches no linkage.

**Criticality** — `critical` | `standard` | `routine`. Declared on every requirement; absence is
a Finding, not a default. Determines which artifacts are required at all, not merely how strong the
evidence must be (D6.5, D20). Routine stops at intent and owes no linkage; standard and critical
propagate to code along `realizes` edges. Criticality is never a property of a directory and may
change through a change without changing claim identity (D21.1).

---

## Facets

**Facet** — one of the three things that can be said about a claim. Missing facets relative to the
declared rigor generate the basic completeness Findings (D3, D20). Incomplete facets, cross-facet
consistency and enumerator machinery generate further findings; the stronger claim that facet
presence generates the whole taxonomy has been partially falsified.

**Intent** — what must be true, over what domain, and how much it matters. Recorded in the spec.

**Mechanism** — what makes it true, and how strongly. Recorded in the design artifact.

**Mechanism identity** — a stable, design-owned id for one atomic enforcement mechanism. It is
independent of a code symbol, so deletion can leave an unresolved declaration rather than erasing
both sides of the relation.

**Evidence** — how we know it is true, and how freshly. Recorded in the verification plan.

**Residue** — everything that belongs to no claim: orientation, danger zones, deliberately broken
corners, what is absent and why. Outside the model, in no check, and underivable by anything.
Named so that the design artifact does not become a dumping ground.

**Facet owner** — whoever is accountable for the sufficiency and freshness of one facet in a
particular organization. `Intent owner`, `mechanism owner` and `evidence owner` name
accountabilities, not required job titles or exclusive authorship rights. Ownership is an optional
layer outside the core model (D3.1, D30).

---

## Evidence

**Evidence** — anything that supports belief in a claim: tests, static rules, type and schema
constraints, DB constraints, model checks, fault injection, canary metrics, production monitors,
manual passes, third-party attestation. *Narrowing:* not evidence in the legal or Bayesian sense;
no weight is combined or accumulated across items.

**Strength** — how far the evidence reaches:

- **Proof** — established by construction over all executions. *Narrowing:* far weaker than the
  formal-methods sense. No obligation is discharged and no semantics is checked; the predicate
  remains prose. A unique index, a type constraint, or a static rule is proof-strength here
  because violation is unrepresentable, not because anything was proved.
- **Demonstration** — held for the executions sampled. All tests, including property tests: a
  wider sample, still a sample.
- **Detection** — we would learn if it stopped holding. Monitors, reconciliation jobs, alerts.
  A claim about the *detector*, never about the property.

**Quantification** — `example` | `universal`: whether evidence checked one case or ranges over
all of them. A property of evidence, not of the claim. This is property-based testing's
example/property cut, named on the *breadth of the evidence* rather than on the predicate under
test — the framework accepts derived enumeration, generation and repeated contention as satisfying
it, and only one of the three is a property test. *Narrowing:* `universal` names the quantifier the
evidence ranges under, not exhaustiveness; a generated or interleaved space is a wider sample and
still a sample (see **Demonstration**). *(revised — the value was `invariant` until D19, which
renamed it because a Floyd or Meyer invariant is a predicate about the system, and this field is
about the evidence. `invariant` is now no value of this field at all. The word survives here only
as the alpha's name for a cross-cutting rule, which in this framework is a claim with a non-default
domain, and in `invariant-breach`, the Finding kind for a member of such a domain that discharges
nothing.)*

**Scope** — `unit` | `component` | `e2e`, defined by what must be *real*, not by how much runs
(D15). Applies to demonstration-strength evidence only: a static rule executes nothing and has no
scope; a monitor has a target.

**Oracle** — how the expected result was obtained: `direct` writes it in the evidence; `golden`
reads a recorded output; `relational` checks a relation among values observed for one case;
`metamorphic` checks a relation across executions connected by an intentional transformation;
`model-based` computes the exact expected result with an independent model; `contract` consults an
agreed interface or protocol. The vocabulary is validated, but it is descriptive and never ranked
or gated. *(revised 2026-08-10 — `relational` separates within-case relations from metamorphic
relations across executions.)*

**Freshness** — what re-establishes an evidence item and how often, plus how it dies silently. A
test is re-established every CI run; an attestation ages out; a monitor whose query broke has
fired zero times for six months and is worse than no monitor, because it is carried on the books.

**Check** — a deliberately enrolled verification method that directly evaluates one or more
product or operational Claims. It is not every native test, rule or analyzer in a repository. Its
implementation linkage identifies executable source; that linkage does not itself say what the
outcome means.

**Evidence Binding** — one repository-owned relation from a Check's atomic terminal outcome to one
Claim aspect. It states the evidence proposition, evidence form, required context, challenge domain
and qualification policy. One Check may have several Evidence Bindings only when the same atomic
outcome honestly bears on every target Claim; independently variable assertions are separate
Checks.

**Qualification** — the reviewed repository decision about whether one exact Evidence Binding is
credible evidence for its Claim aspect under the required context. Its identity includes the Check
fingerprint, binding fingerprint and required context. A Qualification neither records an
execution nor establishes that the Claim is satisfied.

**Claim Judgment** — the reviewed repository decision about the total assurance composition for
one Claim. It considers realizations, mechanisms, guarantees, Evidence Bindings, Qualifications and
residual risk. It is distinct from both a binding-level Qualification and Subject-specific
Assurance State.

**Challenger** — a method that searches for a reason to distrust one Qualification or Claim
Judgment. The proposition determines the role, not the executable brand. Mutation testing, broad
static analysis and qualification-oriented fault injection normally act as Challengers. A
Challenger does not recursively require a Qualification in alpha 2.

**Challenge Result** — one terminal `clean | findings | inconclusive` result targeting the exact
fingerprint of one Qualification or Claim Judgment. `clean` means only that the Challenger found no
objection in its declared search domain. A Challenge Result creates no product evidence; graph
dependencies propagate its effect without fabricating duplicate downstream results.

**Subject** — the exact thing about which execution facts are asserted. A Subject may identify a
developer workspace, CI candidate, released artifact, deployment, or service and bounded monitoring
window. Equality is strict enough that facts for one Subject cannot leak to another.

**Run** — a bounded, provider-neutral execution envelope over one exact Subject. It records its
plan, actual selection, context, provenance and outcomes, and may contain Check executions,
Challenger executions or both. It need not correspond to one native process. A Check executes
inside a Run; a Check never emits a Run.

**Observation** — the one terminal `satisfied | violated | inconclusive` result for a `(Run,
Check)`. It is an execution fact, not an Evidence Binding, Qualification or Claim Judgment.

**Assurance State** — a dynamic conclusion for one exact Subject, derived from repository-owned
decisions and accepted execution facts. New facts may change state or reopen work; they do not
rewrite repository rationale.

**Assurance Service** — the optional durable ledger for accepted Runs, Observations, Challenge
Results, derived Assurance State, gates and exceptional work. It receives provider-neutral facts
and does not parse repository semantics or host provider-specific integrations. Local bundles have
the same semantics, so using the service is not required (D43).

**Adapter** — an explicitly configured provider-family integration. Core selects semantic targets
and supplies a bounded plan; the adapter translates it to native selectors or imports an existing
report, reports actual selection and returns normalized results with native artifact references. An
adapter never interprets the repository model independently.

**Adapter capability** — a stable `<adapter-id>/<capability-id>` identity in one of the semantic
classes `model.extract`, `check.execute`, `check.import`, `challenge.execute` or
`challenge.import`. Provider capability names remain open; Challenger forms are policy rather than
hard-coded core tool kinds.

**Detector test** — a test proving that a detection-strength item actually fires: that the
reconciliation job flags an injected imbalance, that the deletion scan flags a planted record.
Required for every detection item (D4.3). This is what makes liveness claims checkable before
release.

**Residual risk** — what is knowingly not covered, and why that is acceptable. A first-class
field, because with mixed evidence "covered" stops being binary.

---

## Mechanism

**Enforcement strength** — a ladder, strongest first: unrepresentable (type/schema) >
structurally unbypassable (choke point, DB constraint) > centrally applied but opt-in
(middleware) > guard at every site.

The top two rungs **are** proof-strength evidence — strong enforcement is self-evidencing. The
bottom two are enforcement that proves nothing on its own (D7).

**Choke point** — a single place a violation would have to pass through. Contrast with a guard at
every site, which is the design that leaks.

---

## Linkage

**Tag** — a machine-readable source annotation. Realization tags name `(spec-id, scenario-id)`;
mechanism tags name `(design-spec-id, mechanism-id)`. Check implementation linkage identifies a
stable Check without declaring an Evidence Binding in source. Routine Claims owe no linkage.

**`realizes`** — on a production mechanism: this site is on that claim's path. A site may be
application code or declared delivery topology when routing is part of the behavior. It carries no
form; form describes how a Check evaluates a Claim, not a property of a production mechanism.

**`covers`** — the transitional alpha 1 source annotation that directly links a native test to a
Claim. D43 separates Check implementation linkage from repository-owned Evidence Bindings. The
dependent format change removes this tag without treating it as the alpha 2 semantic relation or
providing a compatibility reader.

**`implements-mechanism`** — on production code: the enclosing compiler-resolved symbol implements
the named design mechanism. This derives a binding; it does not replace the independent design
declaration.

**`covers-mechanism`** — the corresponding transitional alpha 1 source annotation for evidence
about a mechanism contract. It does not itself bind a Check outcome to any product Claim.

**Enumerator** — what produces the member set for a claim ranging over a set of sites. Must be
derived from the same source the system is built from — the route table, the DI container, the
type graph. A hand-listed surface is worse than no rule, because it reproduces the very bug the
rule prevents and reports green (D13.1). It enumerates domain members, not the semantic requirements
each member realizes.

**Surface** — a named site domain assembled from independently derived enumerator contributions.
Each contribution binds an area mount to an enumerator. A surface answers which sites are in a
universal domain; it does not say which ordinary business claims those sites realize (D41).

**Realization obligation** — an optional requirement that an ordinary standard or critical claim
have at least one realization in each named area. It constrains architectural participation, not
evidence location or scope, and carries no mandatory role vocabulary (D41).

**Design binding** — the single machine-addressable artifact resolved for a current design
mechanism. It is either explicit for a non-code artifact or derived from one implementation tag.
Resolution establishes existence. Only properties emitted independently—currently index
uniqueness, columns and predicates—can establish more.

**Delivery topology** — the exchange, bindings, durable queues and dead-letter routes that connect
a brokered producer to its consumers. It is a realization site because correct endpoint code with a
missing binding does not realize delivery. Source declarations establish requested topology; a
deployment-side enumerator is needed to establish what an environment actually deployed (D26).

**Exemption** — a deliberate, attributable, reviewable opt-out from an obligation. Fine anywhere;
a silent absence from an obligation is not. An untagged test asserts no Azimuth evidence, so it has
nothing to be exempted from (D20.1).

---

## Findings

**Finding** — one deterministic model-validation result with a stable kind, closed category,
severity, source location, optional Claim and criticality, detail and corrective help. The basic
completeness Findings are missing-facet combinations relative to criticality:

| Facets present | Finding |
|---|---|
| intent, no mechanism | **unrealized** |
| intent, no evidence | **uncovered** |
| evidence, no intent | **dangling tag** |
| mechanism, no intent | **dangling realization** (rogue complexity) |
| intent + evidence below the declared standard | **wrong-form** |

Incomplete facets, cross-facet consistency, agent judgments and **enumerator unsound or underived**
(D13.2) add findings not generated by facet presence. Their existence partially fired D3's recorded
falsifier; they do not by themselves establish a fourth facet.

---

## Tooling and process

**Project** — the complete Azimuth assurance account assembled from one or more repositories and
model sources. Completeness is declared by a project catalog rather than inferred from whichever
inputs happen to be present (D33).

**Repository** — an independent version-control and delivery boundary. It owns areas and may own
model sources, but neither claims nor changes are confined to one repository.

**Area** — a stable namespace for implementation or evidence sources, owned by one repository in a
project snapshot. Its named mounts locate code, tests, migrations or non-code artifacts. It is not a
domain spec and does not mean evidence `Scope: component`. A local workspace uses the same area and
mount vocabulary without a repository field (D41).

**Typed source address** — an extractor-defined address such as a .NET symbol, TypeScript export,
Next route or PostgreSQL index. `(area, address kind, address)` is stable source identity;
repository, mount and path are locators.

**Model source** — the authority that owns a set of intent packages. Model-source ownership
follows intent accountability, not code placement; one source owns a spec even when its
realizations span many repositories.

**Project reference** — a versioned repository-local locator for the singular project catalog and,
optionally, an integration workset. It identifies the current repository so `project locate` can
derive its areas and model sources. It is not a second topology authority.

**Workset** — concrete repository checkouts and pinned manifests used for one assembly. A workset
may be partial. Its presence never redefines which inputs make the project complete.

**Repository manifest** — a versioned, revision-bound observation emitted by one repository. It
names its areas, model-source digests, active and archived changes, producer and typed linkage
records.

**Change authority** — the one repository observation that contains a change id, whether active or
archived. A change may receive work from many repositories but complete assembly permits only one
authority for its proposal and history (D34).

**Execution receipt** — a content-addressed result of composed evidence naming the exact
repository revisions observed. It is distinct from a manual Run bundle about a Subject.

**Project snapshot** — a finalization record over a complete, clean assembly: catalog digest and
area topology, repository revisions, manifest and receipt digests, and the derived model
fingerprint.

**Project acceptance** — the machine-checked transition from one complete accepted-active workset
to one complete tested-archive workset. It proves an unchanged change directory moved within its
singular authority and that no other tracked content entered the archive revision (D34).

**Change** — the temporal envelope from an accepted current model to a proposed target model.
Carries intent deltas, solution design where needed, implementation work and verification
obligations. Proposed facts do not become current facts until completion (D21). A change is a
semantic transition, not a Git branch, merge request, artifact, environment or rollout (D31).

**Archive** — the immutable semantic record of a completed, rejected or abandoned change. A
completed change updates the current facets before it is archived; a rejected or abandoned one
updates none.

**Finalization** — the derived model fingerprint and validation summary for an accepted, applied
change. It gates the mechanical archive move and contains no authored risk decision.

**Validation** — deterministic interpretation of the derived model. `azimuth validate` reports
Findings without executing enrolled Checks.

**Traceability report** — a pure derived view of selected case-level Claims and their ordered
realization source identities. `azimuth report traceability` creates no repository authority or
execution fact.

**Machine tier** — deterministic model validation. Reports structural Findings. Cannot be argued
with, and cannot establish truth.

**Model package** — the physical directory `azimuth/model/<spec-id>/` anchored by `spec.md`, with
optional sibling `design.md`, `verification.md` and `judgments.md`. Declared ids remain semantic
identity; colocation is navigation and creates no assurance relation (D32).

**Agent tier** — the judgment pass: does each realization site establish part of the predicate, is
the covering evidence toothy, is its declared form honest, and is a required behaviour missing
from the spec. A judgment never establishes the claim; its negative verdicts create Findings and
its fingerprint expires when a relation or source it examined changes (D18, D28, D30).

**Export** — the derived model, serialized. Validators, execution planners, dashboards, PR
annotations and the agent tier are all consumers of it; nothing re-parses specs.

**Rollout** — exposure of an accepted artifact across environments or user populations. Normally
outside the change model. A production observation enters change acceptance only when the proposal
declared it necessary evidence before implementation (D31).

**Steel thread** — one scenario carried end to end through every layer before any breadth, so
that the fan-out exists in week one.

**Fan-out** — one claim realized at several sites across components and languages. The reason
specs are organized by domain area rather than by service.
