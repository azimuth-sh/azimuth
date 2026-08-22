# Research: Alpha 2 Evidence Control Plane

Research date: 2026-08-21

This note records repository-discoverable facts used by the exploration. It is not a landscape
survey and does not make proposed alpha 2 behavior current truth.

## Release and authority

The CLI and Assurance Service crates are versioned `0.1.0-alpha.1` in
[`tools/azimuth/Cargo.toml`](../../../tools/azimuth/Cargo.toml) and
[`services/assurance/Cargo.toml`](../../../services/assurance/Cargo.toml). The release history in
[`docs/decisions.md`](../../../docs/decisions.md) records that alpha 1 created a standalone
canonical repository and public contract.

The repository working rules state that there is no backward-compatibility obligation during the
alpha design phase unless an accepted change says otherwise. No inspected active decision creates
such an obligation for alpha 2. This permits a coherent command and format migration, but does not
remove the need to document it.

The active `canonical-development-authority` change is independent work. This exploration can
depend on its eventual accepted release boundary, but cannot alter, finalize or archive it.

## Existing assurance decisions

Four decisions in [`docs/decisions.md`](../../../docs/decisions.md) form the immediate conceptual
predecessor:

- **D38** classifies mutation testing as qualification of evidence rather than direct verification
  of Claims.
- **D39** introduces generic assurance observations with explicit evidence and challenge bindings.
- **D40** separates stable evidence qualification from recurring lifecycle observations.
- **D42** introduces Claim contracts between repository authority and an assurance service.

The derived account in [`docs/framework.md`](../../../docs/framework.md) and the current
[`docs/glossary.md`](../../../docs/glossary.md) already distinguish qualification from recurring
execution. The exploration therefore refines an existing direction rather than introducing
execution evidence from nothing.

The current model is still split in ways that motivate alpha 2:

- CLI observations use evidence or challenge roles in
  [`tools/azimuth/src/model.rs`](../../../tools/azimuth/src/model.rs).
- The service domain has separate `EvidenceDefinition`, `Qualification`, `Observation`,
  `Challenge`, gate and work records in
  [`services/assurance/domain/src/lib.rs`](../../../services/assurance/domain/src/lib.rs).
- Repository verification declarations still describe imported executions through the D39
  observation protocol in
  [`azimuth/formats/verification.md`](../../../contracts/verification.md).

These records demonstrate two structurally different needs: a stable reviewed decision about an
evidentiary edge and repeatable facts about exact execution Subjects. They do not yet provide the
agreed first-class Check, Evidence Binding, neutral Run or exact Challenge target model.

## Current validation command

The current `azimuth check` implementation in
[`tools/azimuth/src/check.rs`](../../../tools/azimuth/src/check.rs) validates model completeness. It
does not orchestrate native tests, analyses or monitors.

Its `HoleKind` set covers model-wide concerns, including declarations, references, evidence,
observations, judgments, change state and federation. That scope is materially broader than a
requirements traceability matrix. The consequential inference is that `azimuth validate` is a more
accurate public verb and that traceability should be an optional derived report rather than the
validator's identity.

This inference changes active terminology and must be carried by an accepted decision before the
derived framework and working rules are updated.

## Current observation and service boundary

The CLI parses imported observations into its in-memory model through
[`tools/azimuth/src/manifest.rs`](../../../tools/azimuth/src/manifest.rs). The model is therefore
not inherently dependent on a deployed service.

The reference Assurance Service is the only built-in durable operational store. Its server writes
Qualifications, Observations and Challenges through immutable-record endpoints in
[`services/assurance/server/src/lib.rs`](../../../services/assurance/server/src/lib.rs), while the
domain derives lifecycle gate reasons and work from their current applicability.

The existing [`services/assurance/README.md`](../../../services/assurance/README.md) explicitly
keeps repository intent authoritative and presents the service as optional. That boundary survives
the exploration: alpha 2 can make the durable unit a Run without making a service mandatory.

The current
[lifecycle API tests](../../../services/assurance/server/tests/lifecycle_api.rs) already exercise
satisfied and violated observations, challenge findings, expiry and qualification staleness. They
are useful migration evidence but do not establish the proposed Run aggregation or temporal
correction rules.

## Adoption findings

The repository dogfood account in
[`azimuth/explorations/drim-dogfood/exploration.md`](../drim-dogfood/exploration.md) records fifteen
generic findings across installation, initialization, discoverability, linkage, CI behavior,
skills, judgment and model clarity. The findings are motivation, not executable dependencies.

Their combined consequence is that alpha 2 cannot be only an internal schema revision. The release
needs a cold-consumer path that demonstrates:

1. installation and version discovery;
2. project initialization and progressive guidance;
3. Claim, Check and Evidence Binding authoring;
4. deterministic validation;
5. local execution or native-report import;
6. CI Qualification Challenges;
7. optional ledger ingestion and Assurance State inspection.

## Execution-system boundary

The repository already treats assurance extensions as external producers rather than core tool
categories in [`docs/assurance-extensions.md`](../../../docs/assurance-extensions.md). The accepted
D39 direction allows mutation, SARIF, load, chaos and monitoring-like inputs to share a normalized
boundary without adding provider-specific core domain types.

The consequential alpha 2 inference is that Azimuth should control semantic selection and
interpretation while native systems retain execution details and full artifacts. A provider-neutral
Run can carry the exact Subject, plan, actual selection, normalized outcomes, provenance and
artifact references without recreating CI, TestOps or telemetry storage.

## Scale inference

Modern storage capacity alone does not decide which results belong in Azimuth. If every discovered
test automatically emitted an Observation, native inventory would become accidental model
authority and create unstable identities even when storage was inexpensive.

Deliberate first-class Check enrollment solves the semantic problem. A separate ledger benchmark
must settle ingest, query, retention and compaction defaults. Storage policy must not decide whether
a Check counts as evidence, and assurance policy must not imply that every accepted Run is retained
forever.

## Research limits

- The exact alpha 2 schemas, fingerprints and temporal rules remain unimplemented.
- No provider adapter protocol has been tested against two structurally different adapters.
- No scale benchmark yet establishes practical retention defaults.
- No clean-room project has completed the proposed alpha 2 consumer journey.
- Existing records are evidence of the current model, not proof that the candidate model is
  sufficient.
