# Assurance extensions

Status: **compact verification packages, independent review records and the current Run ledger implemented; compact Decision Policies and challenge scheduling deferred**.

Azimuth is an evidence control plane, not a catalog of testing and analysis products. Packages extend authoring, extraction and execution over shared Requirements, Scenarios, Checks and inline contributions. Requirement Review assesses definitions and their verification credibility; Requirement Status assesses applicable Check Results for an exact Verification Target or deployment. Machine-executed Verification Challenges search for weaknesses in an exact assessment. No separate Check Review, Method Qualification, Applicability Decision or Requirement Judgment step is mandatory in the compact account. Historical declaration contracts and review records retain their original meanings without granting current positive assurance.

This document records the role and authority boundaries extension work must preserve. Strict wire details remain in the [adapter](../contracts/adapter.md), [launch-plan](../contracts/run-launch-plan.md) and [Run-bundle](../contracts/run-bundle.md) formats.

## Choose the role by proposition

The executable brand does not decide whether an activity is a Check or a Verification Challenge. Ask what one atomic terminal result would establish.

- A load threshold over a declared workload is a Check when the workload, threshold and result directly evaluate a product Requirement.
- A recovery or alert assertion under fault injection is a Check when it directly observes a recovery, durability, isolation or alerting Requirement.
- Fault injection aimed at whether an existing Check notices a defect is a Verification Challenge.
- Mutation testing is normally a Verification Challenge because it attacks Check sensitivity rather than the product predicate.
- Broad static analysis is normally a Verification Challenge because a clean search does not establish product behavior.
- A requirement-specific static rule with an independent product oracle can be a Check.
- Flakiness repetition, test-order randomization and oracle mutation are Verification Challenges because they
  search for reasons to distrust a Requirement Review or Requirement Status assessment.
- A contract or schema assertion is a Check when the accepted or rejected interaction is itself the product predicate.
- A backup restoration or rollback drill is a Check when it directly evaluates a recovery Requirement for an exact artifact or deployment.
- A machine-executed penetration search can be a Verification Challenge because negative search does not imply product satisfaction. Agent-only exploratory investigations are not enrolled Verification Challenge execution in this increment.

One physical execution may perform both roles. A broker-loss exercise could directly evaluate a recovery Check while challenging whether another Check detects the injected fault. The runtime model must preserve the product outcome and the separately targeted challenge outcome rather than collapsing them into one generic success.

Every Check has at least one Evidence Binding to a product or operational Scenario. One Check may bind
to several Scenarios only when its terminal outcome is atomic and honestly bears on each.

## Current repository boundary

The compact extension seam is package-owned authoring and extraction over the shared Check–Scenario graph. `azimuth.surface` declares Surfaces and independent Expectation Sets; `azimuth.network` declares Probes and origins. Packages expose canonical namespaces, descriptors and validators without making providers authoritative for Scenario meaning. See [verification-packages.md](../contracts/verification-packages.md) and [assurance-reviews.md](../contracts/assurance-reviews.md).

The compact account keeps distinct ownership:

- `verification.md` declares Checks, inline contributions, package concepts and machine-executed Verification Challenges;
- source tags identify Check/Verification Challenge implementations and meaningful Requirement or Mechanism realization scope;
- ecosystem extractors emit semantic sites, accountable locations and fingerprints, not reviewer decisions;
- exact Requirement Reviews and Requirement Status records are authored separately and compiled against pinned packets; and
- Runs retain actual execution Verification Targets, Check Results, Challenge Results and provenance.

The older repository declaration lane retains its separately documented Decision Policies and Challenge Plans. It is historical guidance for that grammar, not a mandatory compact review layer. See [the historical review contract](../contracts/history/assurance-reviews-before-claim-health.md).

Ordinary tests, analyzer rules and monitors remain outside Azimuth until deliberately enrolled. This prevents thousands of native test cases from becoming accidental assurance authority. It is independent of storage capacity: the current ledger may retain large execution volumes while the semantic Check graph remains sparse.

Every Requirement in the accepted model under `azimuth/model/` is routine. They therefore have no current
Checks, Evidence Bindings or decisions. The parser, extractor and release suites are ordinary
engineering tests, not Azimuth evidence.

## Current Run exchange

The [`azimuth-run-bundle` version 2](../contracts/run-bundle.md) contract represents one logical Run over an exact source snapshot, artifact or deployment Verification Target; historical Verification Target variants retain their original meanings. It freezes the model and selected Check or Verification Challenge fingerprints, exact Execution Context, planned and actual work, physical activities, ordered attempts, terminal results, content-addressed artifacts, diagnostics and accountable provenance.

Every actually selected Check has one terminal Check Result per selected Scenario. Every selected Verification Challenge target has one separately reduced Challenge Result. A shared activity can contribute to both without making a clean challenge positive product evidence or a violated Check Result a challenge finding. Partial, cancelled and timed-out Runs omit unselected work rather than fabricating results.

`azimuth run verify` checks strict shape, canonical fingerprints, plan/actual agreement, reduction, references and immutable correction chains. `azimuth run inspect` presents the deterministic account and labels current repository authority and Assurance State unresolved. Neither command invokes a provider, reads artifact locators or writes to a service.

## Current bounded adapter boundary

The adapter boundary separates core, which owns repository loading and semantic selection, from an adapter, which translates a frozen selection to provider-native work. Strict `azimuth/adapters.json` configuration pins provider and adapter identity, executable and resource content, description, semantic settings, exact non-secret environment literals, process limits and capabilities. Core never discovers an executable through `PATH`, invokes a shell or inherits the ambient environment.

The semantic capability dictionary is closed:

- `model.extract`;
- `check.execute`;
- `check.import`;
- `challenge.execute`; and
- `challenge.import`.

Provider-family identities, configured `<adapter-id>/<capability-id>` addresses and Challenge forms remain open. One capability may support several classes. One Run uses one configured adapter but may route different selections through several capabilities without combining their semantic results.

The provider-neutral Plan freezes semantic targets, implementations, required Execution Context and finite units. A separate `azimuth-run-launch-plan` freezes the exact Verification Target, planned time, `execute | import` operation, complete Plan and one configured capability route per selection. Substituting a route changes launch identity and the derived Run id.

The public journey is:

```text
azimuth adapter verify [--config <file>]
azimuth run plan --request <file> [--model <dir>] [--standards <file>] \
  [--workspace <file>] [--manifest <file>...] [--config <file>] [--out <file>]
azimuth run execute --plan <file> [--predecessor <bundle>...] \
  [--config <file>] [--out <file>]
azimuth run import --plan <file> --input <id>=<file>... \
  [--predecessor <bundle>...] [--config <file>] [--out <file>]
```

Verification performs the configured description handshake. Planning loads the complete unselected model and accepts Check-only, Challenge-only or mixed strict requests. Check requests name an explicit capability and finite units. Challenge requests additionally name an authored Plan and nonzero candidate cap. The two arrays are required and their combined selection is non-empty.

Compact planning selects exact current Requirement Review or Requirement Status assessment fingerprints and resolves a bounded semantic scope. The mutation execution Verification Target is separate from the review target. Current Stryker integration pins exact batch source input and native mutation statuses/locations; per-mutant exact Verification Targets and mutant-specific product Check Results are unavailable. Future materialized mutants require distinct Verification Targets before their Check Results can support Requirement Status. Source scope uses meaningful Requirement or Mechanism tags rather than duplicate file lists; emitters supply locations and adapters translate them into tool filters. Mutation targets, compilation/analysis dependencies and exercised Checks remain distinct. Missing mappings, exclusions and scope expansion must be recorded rather than silently broadening execution.

Historical authored Challenge Plans retain their policy and schedule semantics. Compact planning does not invent a mandatory Decision Policy, recursive Verification Challenge qualification or positive interpretation of a clean search. The adapter executes the frozen semantic selection and accountable launch inputs; it never loads or reinterprets repository authority.

For every exchange, core stages executable, resource and import-input bytes from the same open streams it hashes and invokes only staged content with a cleared environment. On a supported host, the adapter starts in a fresh process group before its code runs. One configured deadline bounds core request writing, concurrent response and diagnostic reads and core's own wait. Core signals remaining group members on every terminal path and cleans inherited pipes while those processes remain in the group. A host without the required process-group primitive rejects the exchange before spawn.

Authorized adapter code can deliberately call `setsid`, `setpgid` or an equivalent and leave the group. An escaped descendant cannot extend core's wait beyond the deadline, but Azimuth does not guarantee its termination. This is not non-escapable descendant containment, daemon supervision, hostile-code isolation or a filesystem or network sandbox.

Execute and import accept one strict response. Core validates request identity, the repeated description, launch and route provenance, actual selection, reduction and the complete Run bundle before atomic publication. Import provenance retains exact core-computed input digests and sizes; provider paths, URIs and execution ids cannot replace them. Repeatable predecessors must form one verified correction chain, and a response is revision zero or exactly the next revision correcting the complete terminal account.

Adapter, configuration, description, launch, routes and normalizer are correction anchors. Import inputs remain protected by each revision but may change when later bytes from the same native execution arrive through the frozen route. Changing the route creates another Run.

A valid violated Check Result, Challenge finding, partial or cancelled Run, or adapter-returned protocol-valid `timed-out` Run fact exits zero. A host-enforced process deadline is a transport timeout and exits one, as does another transport, semantic, content or identity mismatch. A returned `timed-out` fact is valid only when the complete response arrives inside that host deadline; host timeout publishes nothing. CLI and schema failures exit two. No nonzero result publishes an output bundle.

Challenge Results are exactly `clean | findings | inconclusive`. Clean records only that the
configured search found no objection. Findings attack the pinned assessment and require disposition, correction and reassessment where relevant. They do not fabricate a violated product Requirement, and clean results do not establish a supported Requirement Status.

Adapters are short-lived processes. There is no long-running adapter, service bridge, provider webhook or generic event gateway. Provider-specific event hosting does not enter core or the Assurance Service.

Continuous monitoring must become bounded execution windows. Alert delivery may establish a negative event, but silence is not success unless an enrolled Check also establishes that the measurement and delivery path was complete and healthy for the exact window.

Raw reports and telemetry remain in their systems of record. Run bundles carry content digests, immutable references and enough provenance to preserve their normalized interpretation; standalone verification never dereferences an artifact locator.

The synthetic [Challenge-planning conformance](../experiments/challenge-planning/README.md) proves these planning, outcome, omission, import and mismatch boundaries through public commands only.

## Service and wire boundary

The current `run-ledger` accepts strict Run bundles and independently authored compact review records under project-scoped producer, reviewer and owner credentials. Owners explicitly select a complete authority projection exported with `azimuth assurance authority`; registering a candidate does not accept a change. A snapshot joins that selected authority, immutable reviews and current correction heads for an exact Verification Target. No passing Run fabricates a supported Requirement Status assessment. Stale, rejected, missing and ambiguous support remains visible.

The older claim-contract/project-snapshot service stays isolated, with no compatibility bridge into current Run state. `azimuth export` emits repository model content; the authority command emits its pinned full-model review projection, neither exporting runtime ledger history. Provider-specific event hosting remains outside the ledger. See [run-ledger.md](../contracts/run-ledger.md).

## Acceptance boundary for future extensions

A future provider package or ledger integration is composable only if it preserves all of these properties:

- adding a provider requires no provider-specific semantic type in core;
- unknown schemas, statuses and partial selection fail closed;
- execution is limited to targets selected by core;
- broad analysis creates no implicit product evidence;
- a dual-role fault exercise preserves its two distinct meanings;
- actual-selection mismatch is visible rather than reported as success;
- monitoring silence is not interpreted as a satisfied product result; and
- equivalent normalized bundles work locally and through the optional ledger.

Adapter transport stops before durable ingestion; the separate current ledger supplies authorization, immutable history and Verification-Target-specific compact Assurance State. No cache-validity, automatic expiry, cross-target reuse or historical applicability inference is defined. Compact challenge policies and scheduling remain deferred. Guidance must not simulate those missing authorities or turn checked-in records into fresh measurements.

## Review provenance and publication

Native reviews may retain optional `provenance: {agent, model, reassesses}`. Agent and model labels are null or supplied nonempty names; reassessment is null or an earlier same-target review fingerprint with an earlier review time. Labels do not establish independence or permission. Markdown aggregates preserve reviewer, actual review time, rationale and exact packet dependencies.

Local drafts and experimental executions stay local by default. CI publishes the current package relied on for delivery and relevant adverse facts, preserving original authorship. Review publication requires an authorized reviewer credential matching the record; producer permission cannot publish reviews. ASS retains published immutable records, separately exposes definition Review currentness and exact-target Requirement Status, and never infers acceptance from registration, transport, execution or deployment provenance.
