# Candidate change map: Alpha 2 Evidence Control Plane

This map identifies likely authorities and dependency order, not committed implementation scope.
Proposal work may refine identifiers and boundaries, but each change must carry the exploration
decisions it implements and remain independently reviewable.

## Dependency graph

```text
evidence-control-plane-model
├── verification-evidence-bindings
├── validation-command-surface
└── run-bundle-protocol
    ├── adapter-capability-protocol
    │   └── traceability-challenge-planning
    └── assurance-ledger-runs
        └── adapter-event-gateway (defer unless required by the release slice)

traceability-challenge-planning ───> assurance-ledger-runs

verification-evidence-bindings ─┐
validation-command-surface ─────┼── alpha2-consumer-workflow
adapter-capability-protocol ─────┤
traceability-challenge-planning ─┘

all required implementation changes
        └── alpha2-repository-migration
                └── alpha2-release
```

## Change A — `evidence-control-plane-model`

Establish the public semantic foundation.

Scope:

- revise authoritative decisions and glossary;
- define the two Claim levels;
- define Check, Evidence Binding, Qualification, Claim Judgment, Run, Subject, Observation,
  Challenge, Challenge Result and Assurance State;
- state ownership, identity, cardinality and non-recursive boundaries;
- distinguish repository decisions from execution-ledger facts;
- retire the active `rtm` concept;
- identify obsolete alpha 1 concepts and required migrations;
- update the derived framework account only after decisions are accepted.

Completion evidence:

- structurally different local, CI, mutation, fault-injection, static-analysis and monitoring cases
  can be expressed without changing the definitions;
- terminology has one meaning across decisions, glossary, framework, formats and service language;
- unresolved schema and temporal questions become explicit obligations for dependent changes.

This is the root change. A dependent format or command proposal must not invent its own assurance
semantics.

## Change B — `verification-evidence-bindings`

Reshape repository-owned verification declarations.

Depends on: Change A

Scope:

- define the alpha 2 `verification.md` contract;
- declare first-class Checks and Evidence Bindings;
- express required context, evidence form, challenge domain and qualification policy;
- represent one Qualification per binding;
- define Check implementation linkage for extractors;
- replace source-level coverage conflation;
- validate multi-Claim binding and outcome atomicity rules;
- replace alpha 1 declarations and tags without compatibility aliases or readers.

Completion evidence:

- synthetic cases cover one Check bound to several Claims with separate Qualifications;
- invalid cases reject unbound Checks, independent outcomes hidden in one Check, missing
  implementation linkage and unstable challenge selectors;
- source extractors provide implementation facts without declaring evidentiary meaning.

## Change C — `validation-command-surface`

Make deterministic validation accurately named and navigable.

Depends on: Change A

Scope:

- replace `azimuth check` with `azimuth validate`;
- retire the active `rtm` validator identity;
- group current model holes under coherent validation diagnostics;
- add `azimuth report traceability` only as a derived view;
- update help, initialization guidance, skills, CI examples and diagnostics;
- document the breaking command replacement explicitly without retaining aliases.

Completion evidence:

- every current validation obligation remains covered or is deliberately superseded;
- diagnostic output makes the next corrective action discoverable;
- no active command, help, fixture or current documentation presents the whole model as an RTM.

## Change D — `run-bundle-protocol`

Define provider-neutral execution exchange.

Depends on: Change A

Scope:

- specify Run, Subject, plan, actual selection, outcome, provenance, artifact reference and
  diagnostic schemas;
- support Check and Challenger executions in one Run;
- define one terminal Observation per `(Run, Check)`;
- define exact Challenge Result targeting;
- cover workspace, CI candidate, artifact, deployment, monitoring-window and imported-historical
  Subjects;
- settle retries, sharding, partial execution, cancellation, duplicate import and correction;
- support local validation and inspection without the Assurance Service.

Completion evidence:

- synthetic bundles cover every representative Subject and aggregation edge case;
- dual-role fault execution produces a correctly separated Observation and Challenge Result;
- malformed or materially mismatched actual selection cannot create accepted evidence.

## Change E — `adapter-capability-protocol`

Create the provider integration boundary.

Depends on: Change D

Scope:

- define explicit adapter configuration and `<adapter-id>/<capability-id>` addressing;
- define capability classes for model extraction, Check execution and import, and Challenge
  execution and import;
- specify protocol versioning, invocation, cancellation, timeout, diagnostics, artifacts and result
  validation;
- keep semantic planning in core;
- provide at least two structurally different synthetic conformance adapters;
- define provider-family package and naming conventions.

Completion evidence:

- one adapter executes a plan and another imports an existing native report;
- both pass one conformance suite;
- adapters cannot silently alter planned semantic targets;
- core has no executable dependency on consumer or provider domains.

## Change F — `traceability-challenge-planning`

Implement Qualification and Claim Judgment challenge selection.

Depends on: Changes B and E

Scope:

- select exact Qualification or Claim Judgment fingerprints;
- traverse realization and mechanism linkage to affected Evidence Bindings;
- express qualification standards and required challenge forms;
- map challenge forms to configured adapter capabilities;
- propagate challenged-decision impact to dependent judgments and state;
- define clean, finding, inconclusive, deferred and selection-mismatch behavior;
- expose cost-aware policy without equating deferral with success.

Completion evidence:

- mutation, qualification-oriented fault injection and broad static analysis challenge the intended
  decisions in synthetic fixtures;
- direct product assertions remain Observations;
- traceability gaps and actual-selection drift create visible Findings or inconclusive results;
- no duplicate Claim Judgment challenge is manufactured merely because an upstream Qualification
  was challenged.

## Change G — `assurance-ledger-runs`

Evolve the optional Assurance Service around normalized Runs.

Depends on: Change D; challenge-specific state behavior also depends on Change F

Scope:

- ingest and validate provider-neutral Run bundles;
- persist Runs, Observations, Challenge Results, provenance and artifact references;
- settle authorization, idempotency, correction, revocation, retention and compaction;
- derive current and historical Assurance State deterministically;
- keep Qualifications, Claim Judgments and standards repository-authoritative;
- share bundle validation and state rules with the local path;
- benchmark high-cardinality synthetic workloads.

Completion evidence:

- out-of-order, duplicate, corrected and revoked inputs yield deterministic state;
- state can be queried by Subject and relevant decision fingerprint;
- scale results justify retention and compaction defaults;
- service absence does not prevent local or CI execution and inspection.

## Change H — `adapter-event-gateway`

Provide optional inbound-event hosting without contaminating the ledger.

Depends on: Changes D, E and G

Scope:

- authenticate and route provider-native webhooks;
- prevent replay and bound payloads;
- invoke explicitly configured short-lived import adapters;
- handle retries, duplicate delivery and dead letters;
- submit only normalized provider-neutral bundles to the Assurance Service.

Completion evidence:

- forged, replayed, duplicated, reordered and oversized synthetic events are handled safely;
- provider logic remains outside the Assurance Service;
- adapters remain bounded processes.

Defer this change beyond alpha 2 unless the minimum release story explicitly requires hosted
monitoring ingestion. File-based or CI-driven imports are sufficient to prove the control-plane
boundary.

## Change I — `alpha2-consumer-workflow`

Turn the model into an adoptable cold-consumer journey.

Depends on: Changes B, C, E and F

Scope:

- installation and version discovery;
- initialization and generated guidance;
- progressive authoring of Claims, Checks, Evidence Bindings, Qualifications and Claim Judgments;
- local validation and execution;
- CI Qualification Challenges;
- Run import and optional service ingestion;
- Assurance State inspection;
- language-extractor and agent guidance;
- resolution of applicable dogfood findings using repository-owned synthetic fixtures.

Completion evidence:

- a clean-room synthetic project completes the journey from installation through a challenged
  Qualification and inspected Assurance State;
- documentation distinguishes mandatory alpha 2 capability from extension points;
- no consumer checkout, vocabulary or executable dependency is required.

The workflow may document Change G while retaining a complete service-free path.

## Change J — `alpha2-repository-migration`

Dogfood the accepted alpha 2 model in the canonical repository.

Depends on: every change selected for the release slice

Scope:

- migrate formats, standards, fixtures, extractors, skills, services and active development
  accounts;
- remove alpha 1 formats, aliases and readers rather than maintaining dual behavior;
- remove superseded current terminology and derivable artifacts;
- preserve prior decisions through explicit revisions rather than silent rewriting;
- address every migration failure found by rehearsal;
- confirm build, test, package and publish independence from all consumer repositories.

Completion evidence:

- the repository validates itself under alpha 2;
- synthetic end-to-end workflows and adapter conformance tests pass;
- active change authority remains singular and internally consistent;
- a repository-wide content audit finds no prohibited consumer-domain material or dependency.

The existing `canonical-development-authority` change remains an independent predecessor that must
reach its own honest completion boundary. This exploration does not alter or archive it.

## Change K — `alpha2-release`

Cut and publish the immutable alpha 2 state.

Depends on: Change J and every change selected for the release slice

Scope:

- confirm the final included capability slice selected when this change is proposed;
- close or explicitly defer exploration risks;
- update version, release notes, migration guide and public positioning;
- verify source packages, CLI artifacts, optional service artifacts and installation instructions;
- run the cold-consumer acceptance journey from published artifacts;
- tag and publish only after selected changes are accepted and archived.

Minimum recommended slice:

- the public evidence-control-plane model;
- `verification.md` Checks and Evidence Bindings;
- `azimuth validate` and provider-neutral `azimuth run`;
- an adapter capability protocol with synthetic conformance evidence;
- traceability-driven Qualification Challenges;
- optional Run-ledger ingestion and state derivation;
- a complete service-free local and CI workflow;
- canonical repository migration and cold-consumer evidence.

The generic event gateway and production-grade provider adapters are extension changes unless their
absence would make a claimed release capability false.

The release proposal must select this slice before Change J begins. Release execution then confirms
that the selected slice and its public claims still match the accepted implementation.

Completion evidence:

- a user can install the published alpha, initialize a new project, validate its model, execute or
  import evidence, challenge a Qualification and inspect resulting Assurance State;
- release claims name only shipped behavior;
- publication is reproducible from this repository alone.

All active Claims use routine criticality during this alpha. Existing higher-criticality Claims are
lowered rather than carried with evidence obligations. Each implementation change still owes
ordinary engineering tests and honest command results before acceptance.

## Cross-change falsifiers

- If a passing Observation automatically establishes a Qualification, the decision layers have
  collapsed.
- If a clean Challenger is reported as product evidence, challenge semantics have failed.
- If an adapter must interpret the full repository model, the core/provider boundary has failed.
- If every native test becomes an Azimuth Observation, first-class enrollment has failed.
- If a Run cannot identify its exact Subject and actual selection, it cannot support assurance
  state.
- If the service becomes required for local or ordinary CI use, the optional-service boundary has
  failed.
- If integrating a new provider requires a provider-specific semantic type in core, the capability
  boundary has failed.
- If alpha 2 cannot be installed and exercised from this repository's published artifacts alone,
  the release is not complete.
