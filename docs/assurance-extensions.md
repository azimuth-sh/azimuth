# Assurance extensions

Status: **decision-aware planning and bounded adapter exchange implemented; Run ledger deferred**.

Azimuth is an evidence control plane, not a catalog of testing and analysis products. Alpha 2
implements the repository graph from Checks through Evidence Bindings to Qualifications, plus
total-composition Claim Judgments, Challengers, Challenge Plans, Decision Policies and one current
Challenge Schedule. A strict provider-neutral Run bundle supports service-free verification and
inspection. Explicit short-lived adapter invocation, complete-model Check and Challenge planning,
native execution and exact report import sit beside them. The replacement Assurance Service ledger
remains deferred.

This document records the role and authority boundaries extension work must preserve. Strict wire
details remain in the [adapter](../contracts/adapter.md),
[launch-plan](../contracts/run-launch-plan.md) and
[Run-bundle](../contracts/run-bundle.md) formats.

## Choose the role by proposition

The executable brand does not decide whether an activity is a Check or a Challenger. Ask what one
atomic terminal result would establish.

- A load threshold over a declared workload is a Check when the workload, threshold and result
  directly evaluate a product Claim.
- A recovery or alert assertion under fault injection is a Check when it directly observes a
  recovery, durability, isolation or alerting Claim.
- Fault injection aimed at whether an existing Check notices a defect is a Challenger.
- Mutation testing is normally a Challenger because it attacks Check sensitivity rather than the
  product predicate.
- Broad static analysis is normally a Challenger because a clean search does not establish product
  behavior.
- A claim-specific static rule with an independent product oracle can be a Check.
- Flakiness repetition, test-order randomization and oracle mutation are Challengers because they
  search for reasons to distrust a Qualification.
- A contract or schema assertion is a Check when the accepted or rejected interaction is itself the
  product predicate.
- A backup restoration or rollback drill is a Check when it directly evaluates a recovery Claim
  for an exact artifact or deployment.
- A penetration or exploratory session is a Challenger by default because negative search does not
  imply product satisfaction.

One physical execution may perform both roles. A broker-loss exercise could directly
evaluate a recovery Check while challenging whether another Check detects the injected fault. The
runtime model must preserve the product outcome and the separately targeted challenge outcome
rather than collapsing them into one generic success.

Every Check has at least one Evidence Binding to a product or operational case-level Claim. One
Check may bind to several Claims only when its terminal outcome is atomic and honestly bears on
each. Independently variable results require separate Checks even when one native process produces
them.

## Current repository boundary

The implemented alpha 2 extension seam is repository-owned:

- `verification.md` declares Checks, Evidence Bindings, Qualifications, Challengers and Challenge
  Plans plus one Claim Judgment for every applicable case Claim;
- project standards declare Decision Policies and one `gate | scheduled` Challenge Schedule;
- source uses sparse `ImplementsCheck(<check-id>)` linkage;
- extractors emit implementation identity and source fingerprints, never evidentiary meaning;
- Challenge Plans use seven selector forms for exact current Qualification or Claim Judgment
  fingerprints and preserve every candidate disposition; and
- `azimuth export` version 2 exposes the derived repository graph and Findings.

Ordinary tests, analyzer rules and monitors remain outside Azimuth until deliberately enrolled.
This prevents thousands of native test cases from becoming accidental assurance authority. It is
independent of storage capacity: a future ledger may retain very large execution volumes while the
semantic Check graph remains sparse.

Every Claim in the accepted model under `azimuth/model/` is routine. They therefore have no
current Checks, Evidence Bindings, Qualifications or Claim Judgments. The parser, extractor and
release suites are ordinary engineering tests, not Azimuth evidence. Synthetic fixtures under
`experiments/` do own non-routine Claims with decisions; they exercise the contracts and are not
part of the accepted model.

## Current Run exchange

The [`azimuth-run-bundle` version 1](../contracts/run-bundle.md) contract represents one
logical Run over an exact workspace, CI candidate, artifact, deployment, service or monitoring
window. It freezes the model and selected Check or Challenger fingerprints, exact context, planned
and actual work, physical activities, ordered attempts, terminal results, content-addressed
artifacts, diagnostics and accountable provenance.

Every actually selected Check has one terminal Observation. Every selected Challenger target has
one separately reduced Challenge Result. A shared activity can contribute to both without making a
clean challenge positive product evidence or a violated Observation a challenge finding. Partial,
cancelled and timed-out Runs omit unselected work rather than fabricating results.

`azimuth run verify` checks strict shape, canonical fingerprints, plan/actual agreement, reduction,
references and immutable correction chains. `azimuth run inspect` presents the deterministic
account and labels current repository authority and Assurance State unresolved. Neither command
invokes a provider, reads artifact locators or writes to a service.

## Current bounded adapter boundary

The adapter boundary separates core, which owns repository loading and semantic selection, from an
adapter, which translates a frozen selection to provider-native work. Strict `azimuth/adapters.json`
configuration pins provider and adapter identity, executable and resource content, description,
semantic settings, exact non-secret environment literals, process limits and capabilities. Core
never discovers an executable through `PATH`, invokes a shell or inherits the ambient environment.

The semantic capability dictionary is closed:

- `model.extract`;
- `check.execute`;
- `check.import`;
- `challenge.execute`; and
- `challenge.import`.

Provider-family identities, configured `<adapter-id>/<capability-id>` addresses and Challenge forms
remain open. One capability may support several classes. One Run uses one configured adapter but
may route different selections through several capabilities without combining their semantic
results.

The provider-neutral Plan freezes semantic targets, implementations, context and finite units.
A separate `azimuth-run-launch-plan` freezes the exact Subject, planned time,
`execute | import` operation, complete Plan and one configured capability route per selection.
Substituting a route changes launch identity and the derived Run id.

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

Verification performs the configured description handshake. Planning loads the complete unselected
model and accepts Check-only, Challenge-only or mixed strict requests. Check requests name an
explicit capability and finite units. Challenge requests additionally name an authored Plan and
nonzero candidate cap. The two arrays are required and their combined selection is non-empty.

Planning resolves the fixed union of requested Plans. Only current qualified Qualifications and
current accepted Claim Judgments execute. Every reached candidate counts toward its Plan cap, and
any `missing-decision | stale-decision | rejected-decision | invalid-decision | inapplicable |
unresolved-relation` candidate fails planning. Exact Qualification context, policy-required-form
coverage and the one-adapter boundary must hold. Core validates each explicit capability's
operation class and exact Challenger form; it never chooses or adds a capability. There is no
`--only` or partial-model planning path.

Every Challenge selection freezes its `gate | scheduled` lane, exact target and sorted semantic
scope. Scope contains typed selector anchors and complete decision inputs. Each launch route
projects every source-backed scope item into one exact accountable source, Artifact, enumeration or
surface-member input. Scope affects semantic Plan identity; locator projection affects launch
identity. The adapter does not load or reinterpret the repository model.

For every exchange, core stages executable, resource and import-input bytes from the same open
streams it hashes and invokes only staged content with a cleared environment. On a supported host,
the adapter starts in a fresh process group before its code runs. One configured deadline bounds
core request writing, concurrent response and diagnostic reads and core's own wait. Core signals
remaining group members on every terminal path and cleans inherited pipes while those processes
remain in the group. A host without the required process-group primitive rejects the exchange
before spawn.

Authorized adapter code can deliberately call `setsid`, `setpgid` or an equivalent and leave the
group. An escaped descendant cannot extend core's wait beyond the deadline, but Azimuth does not
guarantee its termination. This is not non-escapable descendant containment, daemon supervision,
hostile-code isolation or a filesystem or network sandbox.

Execute and import accept one strict response. Core validates request identity, the repeated
description, launch and route provenance, actual selection, reduction and the complete Run bundle
before atomic publication. Import provenance retains exact core-computed input digests and sizes;
provider paths, URIs and execution ids cannot replace them. Repeatable predecessors must form one
verified correction chain, and a response is revision zero or exactly the next revision correcting
the complete terminal account.

Adapter, configuration, description, launch, routes and normalizer are correction anchors. Import
inputs remain protected by each revision but may change when later bytes from the same native
execution arrive through the frozen route. Changing the route creates another Run.

A valid violated Observation, Challenge finding, partial or cancelled Run, or adapter-returned
protocol-valid `timed-out` Run fact exits zero. A host-enforced process deadline is a transport
timeout and exits one, as does another transport, semantic, content or identity mismatch. A
returned `timed-out` fact is valid only when the complete response arrives inside that host
deadline; host timeout publishes nothing. CLI and schema failures exit two. No nonzero result
publishes an output bundle.

Challenge Results are exactly `clean | findings | inconclusive`. Clean records only that the
configured search found no objection; it creates neither positive product evidence nor repository
credibility. Every planned Challenge omitted from a partial, cancelled or timed-out Run has one
execution diagnostic scoped to its selection id and no fabricated Result. Scheduled omission is
allowed deferral; gate omission records execution failure. Added or substituted selection is a
mismatch. A challenged Qualification projects an impact edge to its Claim and current Claim
Judgment but never creates a duplicate direct Judgment Challenge Result. `model.extract` is
declared but has no current execution command.

Adapters are short-lived processes. There is no long-running adapter, service bridge, provider
webhook or generic event gateway. Provider-specific event hosting does not enter core or the
Assurance Service.

Continuous monitoring must become bounded execution windows. Alert delivery may establish a
negative event, but silence is not success unless an enrolled Check also establishes that the
measurement and delivery path was complete and healthy for the exact window.

Raw reports and telemetry remain in their systems of record. Run bundles carry content digests,
immutable references and enough provenance to preserve their normalized interpretation; standalone
verification never dereferences an artifact locator.

The synthetic [Challenge-planning conformance](../experiments/challenge-planning/README.md) proves
these planning, outcome, omission, import and mismatch boundaries through public commands only.

## Service and wire boundary

The replacement Assurance Service is deferred with the Run ledger. Its intended authority
is accepted execution facts and derived Subject-specific state, not repository semantics or
provider integrations. The standalone bundle contract supplies the protocol meaning a future
ledger must preserve; it does not authorize or ingest the Run.

The alpha 1 claim-contract and project-snapshot wire remains isolated inside the existing service
boundary until the replacement is implemented. It receives no bridge into the alpha 2
repository graph. There is no assurance-specific export command: `azimuth export` emits only the
version 2 repository model and no runtime ledger records.

## Acceptance boundary for future extensions

A future provider package or ledger integration is composable only if it preserves all of these
properties:

- adding a provider requires no provider-specific semantic type in core;
- unknown schemas, statuses and partial selection fail closed;
- execution is limited to targets selected by core;
- broad analysis creates no implicit product evidence;
- a dual-role fault exercise preserves its two distinct meanings;
- actual-selection mismatch is visible rather than reported as success;
- monitoring silence is not interpreted as a satisfied product result; and
- equivalent normalized bundles work locally and through the optional ledger.

Current adapter transport stops before durable ingestion. There is no authorization, retention,
cache-validity, cross-Subject reuse or Subject-specific Assurance State authority. Active guidance
must not simulate those missing authorities or dynamic assurance with checked-in records.
