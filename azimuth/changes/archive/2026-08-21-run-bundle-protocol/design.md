# Design: Provider-Neutral Run Bundles

## Boundary

A Run is one immutable logical execution over one exact Subject and one exact semantic plan.
Native processes, parameters, shards and retries are subordinate details. A provider rerun with a
new source-execution identity is a new Run; a late report for the same source execution is a
correction of the existing Run.

Version 1 is a standalone exchange contract. Protocol verification proves strict shape, internal
identity, plan/actual agreement, reduction and correction consistency. It does not prove that a
fingerprint is current in a repository, that a Qualification applies or that a Claim has acceptable
Assurance State. Those decisions require the dependent planning and ledger changes.

## Subject and context

The Subject is a tagged union rather than D42's flat optional field bag:

- a workspace or CI candidate names non-empty repository revision and content-fingerprint tuples;
- an artifact names non-empty artifact digest tuples;
- a deployment names environment, deployment, deployed-state fingerprint and artifact digests;
- a service names its exact deployment state; and
- a monitoring window names one or more exact deployed services and a closed half-open Unix-
  millisecond interval.

Absolute paths, branch names, pull-request numbers, candidate refs, tags and dashboards are
provenance or locators, not Subject identity. Workspace and CI-candidate content fingerprints cover
dirty and relevant untracked state. Historical import is `provenance.mode = import` around the
concrete original Subject; an external run id alone is not a Subject.

One Run has one exact string-to-string context. Planned and actual context must be equal as whole
maps. Provider metadata that is not evidentiary context belongs in provenance attributes.

## Plan and actual selection

The plan freezes a complete-model fingerprint and a non-empty union of Check and Challenger
selections. A Check selection contains its Check fingerprint, exact semantic implementation set
and finite work-unit ids. It may occur only once in a Run. A Challenge selection contains one
Challenger fingerprint and one exact `qualification | claim-judgment` target fingerprint. Its
plan-local id gives its terminal result a stable address.

Unsharded work uses one `whole` unit. Parameters and shards become explicit units only when their
complete set is known before execution. Otherwise the provider selects `whole` and remains
accountable for native completeness; the bundle does not turn every native case into a Check.

Actual selection repeats the exact selected semantic entries and work units. A selected Check
always repeats the full planned implementation set; only entries and units may be omitted.
`complete` requires equality of Check/Challenge entries and units. `partial`, `cancelled` and
`timed-out` may retain a subset, but never additional targets, changed fingerprints, substituted
implementations or different context. A material mismatch keeps the file inspectable but
invalidates the bundle for acceptance.

## Activities, attempts and reduction

An activity records one physical execution with timestamps, status, artifact references,
diagnostics and exact string attributes. Check and Challenger attempts reference activities. One
chaos activity may therefore contribute independently to a Check execution and a Challenger
execution without acquiring one ambiguous aggregate outcome.

Each selected work unit contains attempts with contiguous ordinals. A non-completed activity forces
that attempt to be inconclusive. Check reduction is:

1. any violated attempt makes the unit violated;
2. otherwise a final satisfied attempt may recover earlier technical inconclusion; and
3. otherwise the unit is inconclusive.

Any violated unit makes the Observation violated. Satisfaction requires every planned unit to be
selected and satisfied; otherwise the Observation is inconclusive. Challenge reduction is
symmetrical: findings dominate, a final clean attempt may recover technical inconclusion, and clean
requires every planned unit. A findings result references at least one objection diagnostic.

An entirely omitted planned target creates no result because no execution occurred. Every actually
selected Check has exactly one terminal Observation; every selected Challenge has exactly one
terminal Challenge Result. This preserves D43 cardinality without manufacturing facts for missing
work.

## Identity and artifacts

All public identities use SHA-256 over exact versioned canonical JSON envelopes. Set-like arrays
must already have their format-declared order and unique identities; hashing does not repair input.
The bundle carries and the verifier recomputes Subject, plan, actual selection, Run, Observation,
Challenge Result and bundle fingerprints. Result fingerprints exclude artifact locators and
explanatory diagnostic text; the complete immutable bundle fingerprint still protects them.

Artifacts require id, kind, media type, digest and byte size. A locator is either a normalized
bundle-relative path or a URI. Verification never dereferences it. Diagnostics have closed class,
severity and scope variants plus open lower-kebab code and exact string details; they explain facts
but do not determine outcomes.

## Duplicates and corrections

The initial bundle has revision zero and no predecessor. A correction increments revision by one,
names the immediate previous bundle fingerprint and supplies a reason. It is a complete replacement,
not a patch. Run id, Subject, context, semantic plan, native source-execution identity, planned time
and started time are immutable across the chain. If one of those anchors was wrong, the producer
creates a new Run.

Verifying several bundle files is order-independent. Exact fingerprints deduplicate. Missing
predecessors, competing revision content, gaps, forks, cycles or changed anchors invalidate the
set. Timestamps never select a winner. Durable authorization, out-of-order ingestion, revocation,
retention and current-state projection remain ledger concerns.

## Local command surface

`azimuth run verify --bundle <file>...` verifies standalone protocol consistency and correction
sets. A violated Observation, Challenge findings or an explicit valid partial Run still exits zero:
they are execution facts, not protocol failures. Syntax and schema errors exit two; semantic,
selection, reduction or correction Findings exit one.

`azimuth run inspect --bundle <file>... [--format text|json] [--out <file>]` returns a deterministic
derived account and labels model authority unresolved. It performs no provider call, artifact read,
network write or service ingestion. The later adapter change owns plan generation, execute/import
transport and native translation; the ledger change owns ingestion and Assurance State.

Inspect emits that account and exits one when a well-typed bundle set has protocol Findings. It
exits two without an account for malformed JSON, schema or command usage. Protocol-consistent input
exits zero.
