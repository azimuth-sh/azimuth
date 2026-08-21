# Assurance extensions

Azimuth is an evidence control plane, not a catalog of test and analysis products. Core owns model
interpretation, semantic target selection and normalized outcomes. Provider-family adapters own
native schemas, selectors, commands and report import (D43).

## Choose the role by proposition

The executable does not determine whether an activity is a Check or a Challenger. Ask what its
terminal result establishes.

- A load threshold over a declared workload is a Check when workload, threshold and outcome form a
  product oracle.
- A chaos recovery or alert assertion is a Check when it directly observes a product or operational
  Claim.
- Qualification-oriented fault injection is a Challenger because it searches for weakness in an
  evidentiary edge.
- A broad SAST or SARIF scan is a Challenger because a clean search does not establish product
  behaviour.
- Mutation testing is a Challenger because it attacks Check sensitivity rather than the product
  predicate.
- A claim-specific static rule with an independent oracle is a Check because it directly evaluates
  one declared product proposition.
- Flakiness repetition or test-order randomization is a Challenger because it searches for a reason
  to distrust a Qualification.
- A contract or schema compatibility assertion is a Check when the accepted or rejected interaction
  is the product predicate.
- A backup restoration or rollback drill is a Check when it directly observes recovery Claims for
  an exact Subject.
- A penetration or exploratory session is a Challenger by default because negative search does not
  imply product satisfaction.

One physical execution can contain both roles. A broker-loss experiment might directly evaluate a
recovery Check while also challenging whether another Check detects the injected fault. The Run
then contains an Observation and a separately targeted Challenge Result. It does not collapse them
into one generic pass.

Every Check must have at least one Evidence Binding to a product or operational Claim. One Check
may bind to several Claims only when its terminal outcome is atomic and honestly bears on each
aspect. If latency and error rate can vary independently, they are separate Checks even when one
load process evaluates both.

## Provider-neutral boundary

Azimuth core traverses stable realization and mechanism identities to choose Checks,
Qualifications and Claim Judgments. It emits a bounded plan over one exact Subject and later
verifies the provider's reported actual selection. Raw source paths alone are not semantic
selectors.

An explicitly configured adapter then performs one or more bounded capabilities:

- `model.extract` emits provider-neutral model linkage;
- `check.execute` translates selected Checks and runs the provider;
- `check.import` normalizes existing provider results into Observations;
- `challenge.execute` translates exact decision targets and runs a Challenger; and
- `challenge.import` normalizes existing provider results into Challenge Results.

A provider-family package exposes stable `<adapter-id>/<capability-id>` identities. Namespaced
capabilities remain open, while the five semantic classes stay small and closed. Project policy
describes Challenge forms such as mutation or fault injection and maps them to installed
capabilities; core does not hard-code a list of products.

Adapters report native version, context, actual selection and artifact references. They fail closed
on unknown schemas, statuses and partial selection. They never parse `verification.md`, traverse
the Claim graph or decide that a native success covers a Claim. Those remain core and
repository-owned responsibilities.

## Runs and imports

A Run is one bounded envelope over a developer workspace, CI candidate, artifact, deployment, or
service and monitoring window. It may coordinate several native processes as long as Subject,
context, plan, actual selection and outcomes can be interpreted consistently.

Each selected Check produces exactly one terminal Observation: `satisfied`, `violated` or
`inconclusive`. Each Challenger execution produces a `clean`, `findings` or `inconclusive`
Challenge Result targeting one exact Qualification or Claim Judgment fingerprint. A clean
Challenge Result is negative search, never a `satisfied` Observation.

Continuous sources are divided into bounded windows. An Alertmanager webhook can report a negative
event for such a window, but silence is not positive evidence unless a separate enrolled Check
establishes that the measurement and delivery path was complete and healthy. An optional generic
gateway can authenticate a native event and invoke a short-lived import adapter. The Assurance
Service receives only the normalized Run and contains no provider-specific webhook logic.

Raw reports and telemetry remain in their systems of record. Normalized results retain immutable
references and enough provenance to reproduce their interpretation. The optional Assurance Service
can persist Runs durably; a local bundle has identical semantics.

## Repository placement

Repository authority owns Check definitions, Evidence Bindings, Qualifications, Claim Judgments,
standards and residual rationale. A Check implementation may live in a different repository from
the Claim it supports. Extractors report that implementation linkage; they do not declare evidence
coverage.

Execution authority owns Runs, exact Subjects, Observations, Challenge Results and native artifact
references. Checked-in monitoring configuration establishes only the declared configuration. It
does not establish that a deployment loaded the rule, scraped the metric or delivered a
notification; direct Claims about those boundaries need Check Observations for the deployed
Subject.

## Extension acceptance test

A provider integration is composable when it satisfies all of the following:

- adding it requires no provider-specific Rust type in core;
- native schemas, statuses and partial selection fail closed;
- the adapter executes or imports only the semantic targets supplied by core;
- broad analysis creates Challenge Results and no implicit product evidence;
- one dual-role fault Run preserves distinct Observation and Challenge Result meanings;
- actual-selection mismatch is visible rather than reported as a successful Run;
- bounded monitoring imports do not interpret alert silence as success; and
- normalized bundles work both locally and through the optional ledger.

The alpha 2 adapter protocol and its conformance fixtures belong to the dependent adapter change.
Alpha 1 observation import formats and tool-specific commands receive no compatibility reader.
