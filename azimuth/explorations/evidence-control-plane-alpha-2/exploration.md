# Exploration: Alpha 2 Evidence Control Plane

Id: evidence-control-plane-alpha-2
Created: 2026-08-21
Status: direction agreed; open design questions named

## Objective

Shape `0.1.0-alpha.2` as an adoption-led release that positions Azimuth as an evidence control
plane through its public model, formats, tooling and consumer workflow.

The release should:

- make requirement-level and observable case-level Claims explicit;
- make a deliberately enrolled Check a first-class repository concept;
- distinguish Check definitions, implementations, Evidence Bindings, Qualifications, Runs,
  Observations, Challenges, Claim Judgments and current Assurance State;
- provide a provider-neutral Run and adapter boundary for local, CI and operational execution;
- provide a credible cold-consumer path through installation, initialization, agent guidance,
  language linkage, CI and optional service ingestion;
- evolve the first-alpha contract coherently, then publish one immutable alpha 2 state.

This exploration records candidate direction, not accepted product truth. Each implementation
slice still requires its own proposal and authority.

## Boundaries

- The repository remains authority for Claims, Check definitions, Evidence Bindings,
  Qualifications, Claim Judgments, standards and their rationale.
- The optional Assurance Service records Runs, Observations, Challenge Results, derived current
  state, gates and exceptional work. Execution facts do not silently rewrite repository decisions.
- Ordinary tests, analyses and monitors remain outside Azimuth unless an author deliberately
  enrolls them as first-class Checks or configures them as Challengers.
- Mutation testing, broad static analysis and qualification-oriented fault injection are
  Challengers. A fault scenario or analyzer is a Check only when it directly evaluates a declared
  product or operational Claim with an independent oracle.
- Azimuth does not replace TestOps, CI, telemetry, deployment, issue tracking or artifact stores.
  Native reports and raw telemetry remain in their owning systems; Azimuth records compact
  semantic results, provenance and durable references.
- The Assurance Service remains optional. Local and CI workflows can exchange and inspect the same
  normalized Run bundles without deploying it.
- Provider integrations stay outside the dependency-free semantic core through explicit adapter
  capabilities.
- Consumer findings may motivate generic changes, but consumer vocabulary, executable
  dependencies, fixtures and domain-owned intent remain outside this repository.
- Alpha 2 may break alpha 1 formats and commands. Migration must still be explicit and internally
  complete.
- Alpha 2 does not support alpha 1 compatibility. Old formats, aliases and readers are removed
  rather than retained as a parallel path.
- All active requirements for this fast-moving alpha are routine. Implementation still owes
  ordinary engineering tests, but the Azimuth model does not owe Covers relations, Qualifications
  or Claim Judgments until criticality is deliberately raised after the codebase stabilizes.
- This exploration does not authorize implementation, archive an active change or publish a
  release.

## Existing context

The repository facts and consequential inferences are recorded in `research.md`. The most
important context is:

- `0.1.0-alpha.1` is the current public version and no accepted change creates an alpha
  compatibility obligation.
- Azimuth already contains partial concepts for qualification, observation, challenge, gates and
  Claim contracts, but they do not yet form one coherent public control-plane model.
- The existing `azimuth check` performs deterministic model validation rather than executing
  product verification methods.
- The validator currently called `rtm` finds holes across the whole model, so its name no longer
  describes its scope.
- The Assurance Service is the only built-in durable operational store, while the CLI can already
  consume imported observation manifests.
- Existing adoption findings concern installation, initialization, discoverability, linkage, CI,
  agent guidance and the public model.

These facts imply that the public model and formats should be reshaped before the command surface
is polished. They also support a provider-neutral Run bundle, an optional service and explicit
semantic enrollment rather than automatic ingestion of every test result.

## Decisions

### E1 — Use two explicit Claim levels

A requirement-level Claim states the normative product or operational proposition and owns its
criticality. A case-level Claim refines an observable condition that realization and evidence can
address.

Both levels remain falsifiable and addressable. The distinction prevents a broad requirement from
being treated as though one test outcome exhausts it while avoiding a second unrelated concept for
observable cases.

### E2 — Make Check sparse, deliberate and first-class

A Check is a deliberately enrolled verification method. It is not every native test, analyzer or
monitor present in a project. Every Check binds to at least one product or operational Claim.

This semantic enrollment prevents thousands of ordinary test cases from automatically becoming
Azimuth records. Storage scale is an operational concern; intentional evidentiary meaning is the
primary control.

### E3 — Put evidentiary meaning in Evidence Bindings

An Evidence Binding states that one Check outcome bears on one Claim aspect under declared
context. It owns:

- the evidence proposition;
- required execution context;
- evidence form;
- traceability-based challenge domain;
- qualification requirement.

One Check may support several Claims through separate Evidence Bindings only when the same atomic
outcome honestly bears on all of them. Independently varying assertions are separate Checks even
when one native command executes them together.

### E4 — Qualify each Check-to-Claim edge independently

One Qualification evaluates one Evidence Binding: whether the exact Check implementation and
required context are credible evidence for that Claim aspect. A shared execution does not create a
shared Qualification because credibility is relational.

Qualification identity includes the Check fingerprint, Evidence Binding fingerprint and required
context. Shared technical analysis may be referenced by several Qualifications, but alpha 2 does
not introduce a recursively qualified global technical record without a demonstrated need.

### E5 — Keep Qualification, Claim Judgment and Assurance State distinct

A Qualification judges one evidentiary edge. A Claim Judgment evaluates the total assurance
composition for one Claim, including realizations, mechanisms, guarantees, Evidence Bindings,
Qualifications and residual risk.

Assurance State is the dynamic, Subject-specific operational conclusion derived from accepted Runs
and their results. New execution facts can change state or reopen work, but they do not silently
rewrite the repository's Qualification or Claim Judgment rationale.

The distinction answers four separate questions:

1. What must be true?
2. What method can observe it?
3. Why should that method be trusted for this aspect?
4. What did recent execution say about this revision, artifact, deployment or time window?

### E6 — Author Qualifications in the repository and challenge them in CI

Qualification is established during development:

- the machine tier validates structure, fingerprints, applicability and required fields;
- the agent tier evaluates the evidence argument and proposes a Qualification with rationale;
- the evidence owner accepts it through normal review;
- CI runs Qualification Challenges against the candidate revision.

The project qualification standard declares required challenge forms. A Qualification may
strengthen or deviate from that standard only with explicit rationale and residual risk.

A clean challenge means that no objection was found within the declared search domain. It is not
proof. A material finding or inconclusive result blocks acceptance or reopens the affected
decision according to policy.

### E7 — Make `verification.md` own intentional assurance declarations

`verification.md` owns Check identities and definitions, Evidence Bindings, evidence propositions,
required contexts, evidence forms, challenge domains and qualification policy.

Source extractors contribute implementation linkage, conceptually `ImplementsCheck`. They do not
declare that source code covers a Claim. This removes the current conflation between implementation
traceability and evidentiary meaning.

The exact alpha 2 syntax and possible decomposition into adjacent files remain proposal work.

### E8 — Separate Checks from Challengers by proposition

A Check directly produces evidence about a product or operational Claim. A Challenger searches
for a reason to distrust an Evidence Binding's Qualification or a Claim Judgment.

- Mutation testing is normally a Qualification Challenger. Surviving mutations can object to a
  Check's credibility; killed mutations do not create product evidence.
- Broad static analysis is normally a Qualification or Claim Judgment Challenger. A
  claim-specific analyzer with an independent oracle may be a Check.
- Qualification-oriented fault injection is a Challenger. Fault injection that directly observes
  recovery, durability, isolation or alert behavior is a Check.
- One physical chaos execution may have both roles: direct assertions yield Observations, while
  credibility probes yield Challenge Results.

Other possible Challengers include coverage analysis, flakiness repetition, test-order
randomization, oracle mutation, fuzzing, differential or metamorphic probing, dependency and
vulnerability analysis, DAST, penetration testing, red-team exercises, exploratory testing,
telemetry drills, shadow traffic, configuration drift analysis and agent review. The provider does
not determine the classification; the proposition does.

Challengers are not recursively qualified in alpha 2. Their quality is handled through ordinary
tool-release engineering, conformance evidence and review.

### E9 — Target one exact semantic decision per Challenge Result

A Challenger may target either an exact Qualification fingerprint or an exact Claim Judgment
fingerprint. Each Challenge Result targets one such decision.

Challenge selection traverses the traceability graph from an affected realization or mechanism to
the relevant case-level Claim, Evidence Binding and Qualification. Challenge domains therefore use
stable realization and mechanism identities, not raw source paths or an undifferentiated whole
Claim.

The graph propagates the impact of a challenged Qualification to dependent Claim Judgments and
Assurance State. Azimuth does not duplicate a judgment challenge merely because its upstream
Qualification was challenged. A direct product failure is an `Observation: violated`, not a
Challenge.

### E10 — Use one neutral Run envelope

A Run is a bounded execution envelope over an exact Subject. It can contain Check executions,
Challenger executions or both:

- a Check execution yields one terminal Observation for `(Run, Check)` with result `satisfied`,
  `violated` or `inconclusive`;
- a Challenger execution yields a Challenge Result;
- a dual-role physical execution may yield both without conflating them.

A Run is not synonymous with one test process. It is the boundary within which the Subject, plan,
actual selection, context and outcomes can be interpreted consistently.

Representative Subjects include a local developer workspace, a GitHub Actions or other CI revision
or pull-request merge candidate, a released artifact, a deployment, a service and bounded
monitoring window, or an imported historical execution with explicit provenance.

Continuous monitoring is represented as bounded Runs or windows. An alert can provide negative
evidence. Silence is not a satisfied Observation unless a declared Check establishes that the
measurement window was complete and healthy.

### E11 — Use `azimuth validate` and `azimuth run`

The alpha 2 command model is:

- `azimuth validate` for deterministic repository-model validation and Findings;
- `azimuth run` to plan, execute, import, verify and optionally ingest bounded Runs;
- `azimuth report traceability` for an optional derived traceability view.

Checks-only and challenges-only are selection filters on `azimuth run`, not separate orchestration
systems. The active `rtm` command and validator identity are retired because they no longer
describe whole-model validation.

### E12 — Keep semantic planning in core and provider interaction in adapters

Azimuth core traverses the model, selects exact Checks or challenge targets, emits a bounded plan,
and validates the returned actual selection and normalized bundle.

An explicitly configured adapter translates that plan to native selectors and commands, executes a
tool or imports its existing report, reports actual selection, and emits Observations, Challenge
Results or both. It retains references to provider-native artifacts without copying their full data
into Azimuth.

Adapters do not independently parse the Azimuth repository model and are not discovered from an
arbitrary `PATH`.

### E13 — Address provider-family capabilities explicitly

Adapter packages are organized by provider family and expose stable capability identities of the
form `<adapter-id>/<capability-id>`. Alpha 2 defines a small closed set of semantic capability
classes with open namespaced identities:

- `model.extract`;
- `check.execute`;
- `check.import`;
- `challenge.execute`;
- `challenge.import`.

Challenge forms remain extensible and project policy maps them to installed capabilities. Core does
not enumerate individual tools. Provider packages use generic `azimuth-adapter-*` naming rather
than challenge-only naming because one provider may expose several roles.

### E14 — Host inbound events through an optional generic gateway

Adapters remain bounded request/response processes. An optional generic adapter gateway receives
and authenticates provider-native events, invokes the configured short-lived import adapter, and
submits a provider-neutral Run bundle.

For example, an Alertmanager webhook is received by the gateway and normalized by a configured
import adapter; the adapter does not become a daemon. The Assurance Service accepts only the
normalized bundle and does not acquire provider-specific webhook logic. The gateway can be deferred
beyond alpha 2 if file-based and CI-driven imports prove the release boundary honestly.

### E15 — Keep storage policy separate from assurance policy

The optional service is the natural built-in durable ledger for accepted Runs, but the semantic
model does not depend on it. Raw reports and telemetry remain in their source systems.

Retention, compaction, tenancy and deletion are operational concerns. Whether an Observation is
applicable and whether a Claim has acceptable current evidence are semantic concerns. The exact
ledger acceptance and retention policy remains open and must not be inferred from gate policy.

## Rejected alternatives

- **Enroll every test automatically.** Native inventory would become accidentally authoritative,
  create noisy identity and retention problems, and hide evidentiary intent.
- **Let a Check create a Run.** Execution containment is the reverse: the Run contains selected
  Check execution, which yields an Observation.
- **Require one Check per Claim.** This duplicates execution when one atomic outcome legitimately
  bears on several Claims.
- **Give a multi-Claim Check one Qualification.** Credibility can differ for each Check-to-Claim
  edge.
- **Treat mutation, broad analysis or qualification faults as ordinary Checks.** Their direct
  proposition attacks evidence credibility rather than product behavior.
- **Qualify Challengers recursively.** This creates an assurance regress before a second
  structural need has been demonstrated.
- **Infer Qualifications from passing Observations.** Subject-specific outcomes do not establish
  that a method is a credible oracle.
- **Create separate `azimuth check` and `azimuth challenge` executors.** They share planning,
  Subject, provenance, adapters, execution and ingestion; only their semantic outputs differ.
- **Keep the whole-model validator under `rtm`.** The term no longer describes its scope.
- **Make integrations fixed-purpose challenge plugins.** A provider family may extract, execute,
  import and emit both outcome types.
- **Let adapters interpret the repository model.** Semantic selection would diverge between
  providers and couple integrations to format evolution.
- **Auto-discover tools from `PATH`.** Reproducibility, provenance and capability resolution need
  explicit configuration.
- **Make every webhook adapter a daemon.** This duplicates hosting, authentication, lifecycle and
  observability concerns.
- **Put provider webhook logic in the Assurance Service.** That would turn the neutral ledger into
  an integration platform.
- **Require the service.** Local development and CI need a complete provider-neutral path without
  deployed infrastructure.
- **Copy every raw report or telemetry stream into Azimuth.** Source systems already own these
  artifacts.
- **Treat monitoring silence as success.** Broken or missing telemetry can look identical to a
  healthy system without a completeness Check.

## Residual risks

- The combined vocabulary may still be too abstract for first-time users even though each concept
  has a separate job.
- Fingerprint evolution can cause opaque cascading staleness unless semantic and editorial changes
  are separated precisely.
- Authors may hide independent assertions in one Check and create a misleading terminal outcome.
- Missing or coarse traceability can under-select Qualifications during mutation or fault
  challenge planning.
- A provider may execute targets different from the plan; actual-selection drift must be detected.
- Adapter normalization can hide translation bugs without versioned schemas and synthetic
  conformance fixtures.
- Expensive Challenges require risk-based selection, caching or scheduling without treating
  deferral as success.
- Dirty workspaces and environment state make local Subjects difficult to reproduce.
- Positive monitoring evidence requires a complete, healthy collection window tied to the correct
  deployment.
- Large ledgers still need query, retention, compaction, audit, tenancy and deletion design even if
  compact Observation storage is feasible.
- Late imports, corrections and revoked decisions can change current and historical Assurance
  State.
- Local bundles and service-backed workflows may diverge unless they share schemas and transition
  rules.
- Alpha 1 command, tag, format and service migration is broad even without a compatibility promise.
- Users may misread a clean Challenger as positive product evidence.
- An inbound gateway introduces authentication, replay, routing and payload risks.

## Open questions

1. Should the ledger accept every authorized, valid Run independently of lifecycle gate policy?
   The leading hypothesis is yes, with retention and compaction applied separately.
2. Must a Qualification cite particular development Runs or artifacts as its establishment basis,
   and which references belong in Git rather than the ledger?
3. Which canonical inputs invalidate Check, Evidence Binding, Qualification, Claim Judgment,
   adapter capability and Subject fingerprints?
4. How are late, duplicate, corrected, superseded and revoked results ordered for historical and
   current Assurance State?
5. What common Subject schema preserves the distinct identities of workspaces, CI candidates,
   artifacts, deployments and monitoring windows?
6. How do retries, parameterized cases, shards, skips, timeouts and conflicting provider results
   reduce to one terminal `(Run, Check)` Observation?
7. How does policy express required challenge forms, acceptable capabilities, depth, cache
   validity, risk-based sampling and permitted deferral?
8. What selector language maps stable realization and mechanism identities across generated code,
   shared libraries, configuration and infrastructure?
9. What transport, version negotiation, plan schema, artifact reference, cancellation and
   diagnostic contracts form the adapter protocol?
10. What authentication, replay, routing, retry and dead-letter boundary is required if the generic
    gateway enters the release slice?
11. What exact syntax, normalization, schema version and file decomposition should
    `verification.md` use?
12. Which changes require automatic staleness, agent re-evaluation, human re-acceptance or only
    Assurance State refresh?
13. What is the minimum explicit ceremony for enrolling an existing native test or monitor as a
    first-class Check?
14. Which alpha 1 concepts and stored records are deleted, replaced or deliberately left historical
    during the one-way repository migration?
15. What minimum vertical slice makes the alpha 2 evidence-control-plane claim true without
    promising every provider or hosted integration?

## Proposed experiments

- **Local-to-CI qualification challenge:** demonstrate an accepted Qualification, a deliberate
  implementation fault, traceability-based mutation selection and a CI objection to the exact
  Qualification.
- **Dual-role fault execution:** produce an Observation for a recovery Claim and a separately
  targeted Challenge Result from one bounded Run.
- **Static-analysis classification:** exercise a broad analyzer as Challenger and an independent
  claim-specific analyzer as Check.
- **Monitoring import:** demonstrate violated, inconclusive and legitimately satisfied bounded
  windows, including telemetry completeness.
- **Run aggregation:** cover parameterization, retries, shards, skips, timeout, cancellation and
  partial reports.
- **Temporal replay:** import out-of-order and corrected bundles, then revoke a Qualification and
  verify deterministic historical and current state.
- **Scale benchmark:** measure high-cardinality ingest, current-state and traceability queries,
  retention and compaction with synthetic data.
- **Adapter conformance:** exercise one plan-executing adapter and one report-importing adapter
  against the same protocol.
- **Actual-selection mismatch:** verify that fewer, additional or different native targets cannot
  create misleading accepted outcomes.
- **Cold-consumer journey:** install, initialize, declare, validate, run or import, challenge and
  inspect state in a clean synthetic project.
- **Migration rehearsal:** replace the repository's alpha 1 model and fixtures in one direction,
  recording deleted concepts and rejecting old input rather than adding compatibility readers.
- **Gateway threat exercise:** replay, forge, duplicate, reorder and oversize synthetic events.

These experiments are proposed verification obligations for downstream changes. This exploration
does not authorize running implementation experiments by itself.

## Result

Proceed toward alpha 2 as an evidence control plane through the candidate changes in
`change-map.md`. Begin with the public semantic model, then let verification formats, Run exchange,
adapter capabilities, challenge planning and ledger behavior depend on that authority.

The minimum recommended release slice includes the public model, first-class Check declarations,
`azimuth validate`, provider-neutral `azimuth run`, adapter conformance, traceability-driven
Qualification Challenges, optional Run-ledger ingestion, a complete service-free local and CI
workflow, repository migration and cold-consumer evidence.

The generic event gateway and production provider adapters remain extensions unless omitting them
would make a specific release claim false. Publication requires separately proposed, implemented,
accepted and archived changes; approval of this exploration authorizes none of those steps.
