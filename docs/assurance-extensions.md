# Assurance extensions

Status: **Run exchange implemented; provider and ledger protocols deferred**.

Azimuth is an evidence control plane, not a catalog of testing and analysis products. Alpha 2
implements the repository graph from Checks through Evidence Bindings to Qualifications, plus
Challengers and Challenge Plans. D46 also implements a strict provider-neutral Run bundle and
service-free verification and inspection. Provider invocation, current decision-to-Run planning
and the replacement Assurance Service ledger remain deferred.

This document records the role boundary future extension work must preserve. It is not adapter
configuration or invocation guidance.

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
  Plans;
- source uses sparse `ImplementsCheck(<check-id>)` linkage;
- extractors emit implementation identity and source fingerprints, never evidentiary meaning;
- Challenge Plans select exact current Qualification fingerprints through model relations; and
- `azimuth export` version 2 exposes the derived repository graph and Findings.

Ordinary tests, analyzer rules and monitors remain outside Azimuth until deliberately enrolled.
This prevents thousands of native test cases from becoming accidental assurance authority. It is
independent of storage capacity: a future ledger may retain very large execution volumes while the
semantic Check graph remains sparse.

All active Claims in this repository are routine. They therefore have no current Checks, Evidence
Bindings or Qualifications. The parser, extractor and release suites are ordinary engineering
tests, not Azimuth evidence.

## Current Run exchange

The [`azimuth-run-bundle` version 1](../azimuth/formats/run-bundle.md) contract represents one
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

## Deferred provider boundary

D43 accepts a separation in which core selects semantic targets and a provider-family
adapter translates those targets to native selectors. The adapter must report what actually ran
and produce the provider-neutral Run account with references to native artifacts. It must not parse
the repository model or decide evidentiary meaning independently.

Provider packages will expose stable `<adapter-id>/<capability-id>` identities. The semantic
capability classes are `model.extract`, `check.execute`, `check.import`, `challenge.execute` and
`challenge.import`; provider-namespaced capabilities remain open. These names define the accepted
dictionary, not a currently invocable adapter protocol.

The adapter boundary is not a current public protocol. The following remain for dependent changes:

- adapter discovery, capability negotiation and invocation;
- translation of selected Check and Challenger targets;
- generation of plans from current repository and decision fingerprints;
- provider execution and native report import;
- inbound event authentication and bounded adapter invocation; and
- application of runtime facts to Subject-specific Assurance State.

No long-running adapter is implied. A later adapter may be a short-lived process around a native
command or report, while a generic gateway may eventually authenticate inbound events and invoke a
bounded adapter. Provider-specific webhook logic must not enter the Assurance Service.

Continuous monitoring must become bounded execution windows. Alert delivery may establish a
negative event, but silence is not success unless an enrolled Check also establishes that the
measurement and delivery path was complete and healthy for the exact window.

Raw reports and telemetry remain in their systems of record. Run bundles carry content digests,
immutable references and enough provenance to preserve their normalized interpretation; standalone
verification never dereferences an artifact locator.

## Service and wire boundary

The replacement Assurance Service is deferred with the Run ledger. Its intended authority
is accepted execution facts and derived Subject-specific state, not repository semantics or
provider integrations. The standalone bundle contract supplies the protocol meaning a future
ledger must preserve; it does not authorize or ingest the Run.

D42's version 1 claim-contract and project-snapshot wire remains isolated inside the existing
service boundary until the replacement is implemented. It receives no bridge into the alpha 2
repository graph. There is no assurance-specific export command: `azimuth export` emits only the
version 2 repository model and no runtime ledger records.

## Acceptance boundary for future extensions

A future provider integration is composable only if it preserves all of these properties:

- adding a provider requires no provider-specific semantic type in core;
- unknown schemas, statuses and partial selection fail closed;
- execution is limited to targets selected by core;
- broad analysis creates no implicit product evidence;
- a dual-role fault exercise preserves its two distinct meanings;
- actual-selection mismatch is visible rather than reported as success;
- monitoring silence is not interpreted as a satisfied product result; and
- equivalent normalized bundles work locally and through the optional ledger.

Until the adapter and ledger protocols exist, active guidance stops at standalone bundle
verification and inspection rather than simulating provider execution, ingestion or dynamic
assurance with checked-in records.
