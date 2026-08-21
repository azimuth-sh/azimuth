# Change: run-bundle-protocol

Status: accepted and complete

## Problem

D43 defines a Subject-bound Run but leaves no exchange format. An adapter, local command or future
ledger therefore has no common account of the semantic plan, actual selection, retries, shards,
terminal Observations, Challenge Results or immutable corrections. Reusing the isolated D42
service wire would restore Evidence Definition authority and would not represent Check and
Challenger executions in one envelope.

## Outcome

Azimuth has one strict provider-neutral Run bundle version 1. A bundle freezes one exact Subject,
one exact semantic plan, the actual selected subset, execution activities, normalized attempts,
one terminal Observation per actually selected Check and one exact Challenge Result per actually
selected Challenger target.

Canonical SHA-256 identities make Subjects, plans, selections, results, bundles and correction
chains deterministic. `azimuth run verify` validates the standalone protocol and a set of bundle
revisions; `azimuth run inspect` presents the same derived account without an Assurance Service.

## Scope

In scope:

- define the strict `azimuth-run-bundle` JSON version 1 contract;
- define exact workspace, CI-candidate, artifact, deployment, service and monitoring-window
  Subjects;
- keep historical import as provenance around its original exact Subject;
- freeze complete model, context, Check, implementation, Challenger and decision fingerprints in
  one semantic plan;
- distinguish planned selection from the provider-reported actual semantic subset;
- represent finite work units, ordered retries and physical activities without enrolling native
  inventory automatically;
- reduce Check and Challenger attempts deterministically to Observations and Challenge Results;
- define partial, cancelled and timed-out terminal Runs without fabricating results for omitted
  work;
- define content-addressed artifacts, diagnostics and accountable execute/import provenance;
- derive canonical Run, Subject, plan, selection, result and bundle fingerprints;
- validate idempotent duplicate bundles and linear immutable full-replacement corrections;
- add service-free `azimuth run verify` and `azimuth run inspect`; and
- add a synthetic conformance experiment for all Subject and aggregation cases.

Out of scope:

- generating a plan from the repository model;
- adapter configuration, capability discovery, native selector translation or process invocation;
- importing provider-native reports through product-specific readers;
- resolving a Claim Judgment target or deciding current Qualification applicability;
- treating protocol validity, a satisfied Observation or a clean Challenge as Assurance State;
- Assurance Service storage, API migration, authorization, revocation, retention or compaction;
- event gateways, webhooks or long-running adapters;
- package-version or release-authority changes; and
- any alpha 1 observation, Evidence Definition or service-wire compatibility path.

## Affected claims

Add five routine requirements under `framework/run-bundle-protocol`:

- every Run has one exact typed Subject and accountable provenance;
- planned and actual semantic selection are explicit and fail closed on substitution;
- retries, shards and partial execution reduce to exactly cardinal terminal results;
- bundle identity, duplicates and corrections are deterministic and immutable; and
- the protocol can be verified and inspected locally without adapters or a service.

The requirements contain fifteen case-level Claims. They owe no Azimuth evidence, Qualification or
Claim Judgment while routine. The bundle fixtures and component suites are ordinary engineering
tests.

## Completion conditions

- The strict parser rejects unknown or duplicate fields, unsafe numbers, invalid ids, invalid
  timestamps, unstable set members and non-canonical fingerprints.
- Every supported Subject distinguishes mutable display locators from exact content identity;
  monitoring windows are closed half-open intervals over one immutable deployment state.
- Historical imports retain a concrete Subject and identify the native source execution through
  import provenance.
- A non-empty plan pins the complete model fingerprint, exact required context, Checks and
  Challenger decision targets; each Check occurs at most once.
- Actual context equals required context. Actual targets and work units are an exact subset of the
  plan, while a selected Check repeats its complete planned implementation set. Completed Runs
  require entry and unit equality; additions or substitutions invalidate the bundle.
- One execution record and terminal Observation exists for each actually selected Check. Challenge
  plan-local ids and semantic Challenger/target tuples are both unique, and one execution record and
  Challenge Result exists for each selected entry. Omitted planned work creates no synthetic result.
- Violations and findings survive retries. A later decisive attempt may recover earlier technical
  inconclusion, while missing units and unfinished selected work reduce to inconclusive.
- A shared physical activity can yield an Observation and a separately targeted Challenge Result
  without either result implying the other.
- Content-addressed artifact locators are never dereferenced during verification, and diagnostic
  messages do not determine semantic outcomes.
- Exact bundle replay is idempotent. A changed bundle for one Run is accepted only as the next
  full revision naming its immediate predecessor; gaps, forks, cycles and changed anchors fail.
- `azimuth run verify` exits 0 for a protocol-consistent violated, findings, partial or cancelled
  Run, 1 for semantic or cross-record invalidity, and 2 for malformed JSON, schema or usage.
- `azimuth run inspect` emits deterministic text or JSON and never labels standalone protocol
  validity as current model acceptance.
- Plan, execute, import and ingest Run subcommands remain unknown until dependent changes own them.
- All current framework requirements remain routine, and the complete Rust, conformance, release
  isolation and composed-model suites pass.
