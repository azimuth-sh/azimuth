# Exploration: Continuous assurance service

Id: continuous-assurance-service
Status: direction agreed; lifecycle boundary validated

## Objective

Determine whether Azimuth should provide its own service for collecting CI and production
observations, deriving lifecycle assurance state, and returning actionable findings to repository
workflows without making the standalone CLI or repository model depend on a proprietary platform.

## Boundaries

- Preserve the repository as authority for durable intent, design, evidence definitions and
  semantic judgment rationale.
- Preserve the deterministic, offline-capable CLI and provider-neutral observation protocol.
- Treat execution records as immutable facts about an exact revision, artifact, deployment and
  environment; do not turn Git into a telemetry database.
- Integrate runners, TestOps, delivery and observability systems through adapters rather than
  rebuilding them.
- Do not design a generic CI engine, deployment orchestrator, monitoring store, issue tracker or
  ALM replacement.
- Validate a narrow end-to-end slice before choosing a production database, tenancy model or
  managed-service architecture.

## Existing context

- D39 introduced provider-neutral assurance observations with `evidence` and `challenge`
  bindings. The conformance experiment demonstrated mutation, SARIF, load, chaos and federated
  Prometheus inputs without adding tool-specific Rust domain types.
- Current judgments fingerprint observation instances. That is conservative for bounded change
  evidence but would cause unnecessary re-judgment for every recurring CI or production run.
- Repository finalization can assemble exact repository revisions, but no durable execution ledger
  currently binds observations, judgments and gate decisions to built or deployed artifacts.
- Routine claims deliberately owe no Azimuth linkage or judgment. A service must not make the
  OpenSpec-like routine path heavier.
- The framework and CLI are intended to be open and portable. An optional service may extend them
  but must not become a hidden semantic authority.

## Findings

### F1 — The category exists, but its capabilities are fragmented

Assurance-case tools own claim/evidence argumentation; ALM tools own requirements and test
traceability; TestOps tools own executions; delivery platforms own release gates; attestation
systems own provenance. No public system found in this research combines Azimuth's progressive
criticality, source-native linkage, agent judgment, multi-repository assembly and lifecycle
observations. See `research.md`.

### F2 — Observation, challenge and judgment have mature prior art

SACM already models claims, evidence provenance and timing, evidentiary support and challenge,
observations and resolutions. ETB and ACCESS research continuous and runtime assurance cases.
Azimuth should claim novelty in its developer-native composition and cost model, not invention of
the individual assurance concepts.

### F3 — Existing products validate the service need

Kosli's flows, trails, artifact fingerprints, attestations, evidence vault and environment
snapshots closely resemble the proposed execution ledger. Allure TestOps and ReportPortal show why
executions need a durable context outside Git. Harness shows that production observations can gate
and roll back deployments. These are evidence that the application boundary is useful, not reasons
to copy any one product's ontology.

### F4 — Owning the service protects semantic independence, not infrastructure independence

An Azimuth-owned reference service prevents claim identity, evidence roles, judgment freshness and
progressive criticality from being forced into another vendor's control model. It should still use
open standards and existing stores where useful. Owning a proprietary signature envelope,
telemetry format, CI runner or report archive would add cost without strengthening the semantic
core.

### F5 — Repository authority and execution authority must remain separate

Repositories declare what should count: claims, evidence definitions, bindings, workloads,
thresholds, oracles and qualification rationale. The service records what happened: exact subject,
configuration, environment, time, result, report and expiry. A lifecycle gate is derived from both;
neither side alone establishes readiness.

### F6 — Qualification and execution are distinct decisions

An agent can qualify an evidence definition before it runs: if this test executes under the named
conditions and satisfies its oracle, it is credible evidence for the claim. An observation then
records whether that happened. Ordinary successful repetitions should be machine-evaluated;
changed definitions, findings, failures, incompatible context or unresolved subjects return to the
agent tier.

### F7 — Feedback must be explicit and reviewable

Expiry normally requests a rerun. A violated assertion closes a gate and creates diagnosis work. A
challenge finding creates a judgment work item. Environment drift may propose a verification
change. Repeated production evidence may motivate a new claim. The service may open an issue or
draft change, but must not silently rewrite accepted intent or manufacture a sound judgment.

### F8 — This is an SDLC assurance layer, not an SDLC replacement

Azimuth can carry one claim identity from exploration and specification through realization,
verification, release and operation, then return findings to a later change. That places it in SDLC
territory. Its bounded responsibility remains the justification of claims for exact source,
artifacts, deployments and lifecycle stages; planning, source hosting, execution and telemetry
remain in their established systems.

### F9 — Technical and semantic ownership are different

Platform Engineering or Engineering Effectiveness is the natural operational owner of the service.
An Assurance Lead or Quality Architect owns evidence standards and the method. Product teams remain
accountable for their claims, implementation, evidence and acceptance; SRE and security own the
observations from their domains. Assigning all truth to either QA or the platform team would create
a bottleneck or a context-free control system.

## Decisions

- **E1 — Build an Azimuth-owned assurance service.** Do not make Azimuth's lifecycle semantics
  depend on another product's decisions or availability. Existing products remain integration
  targets and comparative controls.
- **E2 — Keep the service optional.** The open CLI, formats and repository workflow remain useful
  without a server. Teams with only local and ordinary CI evidence owe no service deployment.
- **E3 — Keep durable semantics in repositories.** The service stores executions and derived gate
  decisions; it does not become authority for claims, design, evidence definitions or accepted
  judgment rationale.
- **E4 — Build a semantic service, not another delivery platform.** Reuse CI, TestOps,
  observability, artifact and deployment systems through adapters. Azimuth owns claim-aware
  qualification, applicability, freshness, worklists and lifecycle decisions.
- **E5 — Start with a reference vertical slice.** Prove reusable evidence qualification and two
  lifecycle stages before designing the general platform. The slice must run locally and be
  replaceable by another implementation of the same protocol.
- **E6 — Adopt standards at the boundaries.** Evaluate SACM export, OSLC lifecycle links and
  in-toto/DSSE or SCITT-compatible signed receipts. Standards are interoperability seams, not
  replacements for Azimuth's smaller developer-facing model.
- **E7 — Use federated organizational ownership.** An Assurance Platform owner operates the
  service and protocol, while domain teams retain semantic accountability. The service must expose
  the basis of decisions rather than acting as an unaccountable centralized approver.

## Rejected alternatives

- **Use Kosli or another product as Azimuth's required backend.** Fastest initially, but it makes
  Azimuth's semantic evolution and open-source reference workflow contingent on an external
  product. Those products should instead be adapters and experiment controls.
- **Commit every CI and production observation to Git.** This creates revision self-reference,
  continuous churn, retention and privacy problems, and makes source history a poor telemetry
  store.
- **Build a complete platform first.** Authentication, tenancy, retention, integrations,
  dashboards and managed operations could consume the project before qualification and lifecycle
  gating are proven.
- **Let the service modify accepted artifacts automatically.** Execution outcomes are facts;
  changing intent and semantic judgments requires accountable review.

## Open questions

- Does reusable qualification require a new core record, or can it remain a judgment over a stable
  evidence site and verification definition?
- Which exact subject tuple is sufficient: repository revisions, project snapshot, artifact
  digest, deployment id, environment, cohort and observation window?
- Which first production input provides the strongest validation without building a monitoring
  system: Prometheus query receipt, canary health check or alert-delivery receipt?
- Which parts of the reference service should be open source, and which managed capabilities, if
  any, form a commercial product?
- What signing, retention, access-control and privacy guarantees are required before accepting
  production evidence?

## Result

Proceed with an Azimuth-owned reference service, but begin with the qualification-to-observation
vertical slice in `change-map.md`. The service earns a broader product scope only if that slice can
reuse one qualified definition across executions, gate CI and production independently, and feed a
precise work item back to a repository without changing accepted intent automatically.

Experiment A passed on 2026-08-12. Eight deterministic cases reused one qualification across two
CI revisions, confined production evidence to an exact subject, and produced focused work for
expiry, violation, drift, context mismatch and challenge findings. The reference service may now
implement the surviving boundary without revisiting the repository/service authority split.
