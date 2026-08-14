# Candidate change map: Continuous assurance service

This map identifies dependency order, not committed implementation scope. Each change must remain
independently reviewable and carry the exploration decisions it implements.

## Experiment A — Qualification and repeated execution

Status: passed 2026-08-12

Demonstrate one stable evidence definition qualified once, two CI observations over different
revisions, one production observation, expiry, a violated result and a precise repository work item.
Compare repository churn and judgment count against the current observation-fingerprinting model.

This experiment decides whether qualification is a core record, a judgment role, or a derived view.
It precedes service implementation because the persistence schema must not freeze an unresolved
semantic distinction.

Result: qualification is a stable semantic record over an evidence-definition fingerprint;
observations are immutable execution records; lifecycle gates and exceptional work are derived.

## Change B — Qualification and applicability protocol

Depends on: Experiment A

Define the smallest provider-neutral records for:

- stable evidence-definition identity and qualification;
- execution subject: project snapshot, revisions and artifact digest;
- optional deployment, environment, cohort and window;
- observation applicability, expiry and outcome;
- derived lifecycle-gate result;
- exceptional conditions that require agent re-judgment.

The CLI must still accept local files and remain usable without a service.

## Change C — Local reference ledger

Depends on: Change B

Implement an open, single-project reference service that ingests immutable observation envelopes,
validates identity and schema, stores report references and derives current gate state. Provide a
read API that materializes the same manifests consumed by the CLI. Do not add tenancy, dashboards
or a general workflow engine.

## Change D — CI lifecycle slice

Depends on: Change C

Publish a component or load execution against an exact candidate revision and artifact, evaluate a
merge or release gate, and prove that an ordinary successful rerun does not require a new semantic
judgment. A changed definition and a challenge finding must create focused judgment work.

## Change E — Production lifecycle slice

Depends on: Change C, Change D

Ingest one bounded production receipt from an existing monitoring source, bind it to the deployed
artifact and cohort, evaluate a canary or rollout gate, expire it, and retain the immutable decision
history. Raw telemetry remains in the monitoring system.

## Change F — Repository feedback

Depends on: Change D, Change E

Generate repository-facing work items for expiry, violation, challenge findings, context drift and
unresolved subjects. Demonstrate that successful renewals create no repository churn and that no
feedback path silently edits accepted intent or judgments.

## Change G — Integrity and operational hardening

Depends on: Changes C–F

Only after the semantic slices pass, decide signing envelope, retention, authorization, tenant
isolation, backup and recovery, audit export, scale targets and managed deployment. Compare
in-toto/DSSE, SCITT-compatible receipts and private storage rather than inventing a cryptographic
format by default.

## Acceptance falsifiers

- If repeated successful runs require repository commits or agent re-judgment, the lifecycle split
  has failed.
- If the service becomes authoritative for claim meaning, the repository/service boundary has
  failed.
- If a run cannot be unambiguously tied to exact source, artifact and deployment context, it cannot
  gate a lifecycle stage.
- If ordinary routine changes acquire service configuration or observation obligations, progressive
  assurance has regressed.
- If integrating one new producer requires a service-core domain type, D39's extension boundary has
  regressed.
- If existing evidence stores can provide the same semantic result with a thin adapter at
  materially lower cost, reassess which portions Azimuth must own.
