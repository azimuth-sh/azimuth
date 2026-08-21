# Change delivery in the evidence control plane

Status: **operating guidance**. Parser contracts and accepted decisions remain authoritative.

Azimuth separates repository-owned intent and reviewed meaning from execution facts. During the
fast-moving alpha every current framework Claim is routine. Changes therefore use ordinary
engineering tests and release gates without enrolling those tests as Azimuth evidence.

## Accountabilities

| Accountability | Decides |
|---|---|
| Intent owner | what must be true and its consequence |
| Mechanism owner | what makes it true and which boundaries matter |
| Evidence owner | what would justify belief if the Claim later becomes non-routine |

These are accountabilities, not job titles. An agent is an authoring and review instrument; the
person accepting its output remains accountable for the result.

## Current alpha sequence

### 1. Explore uncertainty

Use an exploration before commitment when a topic spans several changes, crosses an unfamiliar
boundary or still has unresolved product choices. Persist only shared decisions and a bounded
change map. Research does not silently become current framework authority.

### 2. Propose one transition

Create one change id with singular authority. State the problem, outcome, in-scope and out-of-scope
work, affected Claim ids and completion conditions. Carry explicit exploration decisions when
applicable.

All new or changed framework Claims remain routine until a later accepted change deliberately
raises criticality after the codebase stabilizes. Criticality follows consequence rather than
implementation size, but this repository currently chooses the routine boundary to avoid
manufacturing immature assurance decisions.

Add a change `design.md` only when alternatives, failure modes, migration order or boundaries need
review. Add `work-packages.md` when work can be delegated safely. `plan.md` remains an
implementation sequence for the change; it is not a verification artifact or test inventory.

### 3. Freeze shared contracts

Before parallel work, freeze public formats, identities and ownership boundaries. Validate
`work-packages.md`; every package declares dependencies, non-overlapping owned paths, objective and
engineering checks. Workers do not finalize, archive or edit shared change state.

### 4. Implement observable behavior

Implement product behavior, structural mechanisms, telemetry and testability seams inside the
approved boundary. Add Realizes only where a production site genuinely establishes part of a
non-routine case-level Claim. Routine Claims do not require source linkage.

Ordinary unit, component, integration and release tests remain normal software. A passing test is
useful engineering feedback, but it is not an Azimuth Check merely because it executes in CI.

### 5. Update current facets

Apply accepted intent deltas to package `spec.md`. Distil only mechanisms that now exist into
current `design.md`. Because every current Claim is routine, do not create package
`verification.md` files, Checks, Evidence Bindings or Qualifications for this alpha transition.

A future non-routine change will use the D45 graph:

```text
Check -> Evidence Binding -> Qualification
```

A Check has one atomic terminal proposition. Each binding relates that Check to exactly one
case-level Claim and owns form, exact required context, challenge domain and policy. The binding id
is also its sole Qualification id. One Check may bind to several Claims and one Claim may receive
several Checks.

Source then uses `ImplementsCheck(<project-global-check-id>)`. It never declares Claim identity,
form, context or Qualification. Unmarked tests remain outside the graph.

### 6. Validate the repository account

Emit fresh language manifests and run:

```text
azimuth validate --manifest <manifest>...
azimuth report traceability
azimuth export --out model.json
```

Validation reports categorized Findings. Traceability is a derived, deterministic view over
case-level Claims and stable graph relations. Export writes model version 2. None of these commands
executes native tests or creates an execution fact.

Run focused engineering tests while iterating, then affected component and composed suites.
Enumerated surfaces must also exercise their real enumerator and a temporary untagged negative
member before acceptance.

### 7. Record the outcome

Complete plan and work-package statuses. Write `outcome.md` with departures and residual decisions.
Framework experiments may record measurements of Azimuth itself; ordinary product changes do not
owe that section.

### 8. Finalize and archive

Confirm completion conditions, current facet updates, fresh manifests and all required engineering
checks. Run `azimuth change finalize`, then archive only after the finalization fingerprint is
current. These commands do not create Git commits, provider executions or deployment records.

For federation, retain complete accepted-active and tested-archive worksets. The singular authority
repository owns the content-preserving archive move. `azimuth project accept-change` verifies the
two complete accounts and emits a snapshot; a local account cannot substitute for project
acceptance.

## Run exchange and deferred execution plane

D46 implements a standalone [`azimuth-run-bundle` version 1](../azimuth/formats/run-bundle.md)
exchange. A Run binds one exact Subject and may contain Check executions, Challenger executions or
both. Its plan and actual semantic selection are explicit, and its retries and work units reduce to
one Observation per actually selected Check and one Challenge Result per selected Challenger
target. Challengers search for objections to Qualifications or later Claim Judgments; a clean
result is not positive product evidence.

Use the service-free commands to verify or inspect an already normalized bundle or correction set:

```text
azimuth run verify --bundle <file> [--bundle <file> ...]
azimuth run inspect --bundle <file> [--bundle <file> ...] [--format text|json] [--out <file>]
```

These commands validate the protocol account only. They do not establish that repository
fingerprints remain current, apply execution results to Assurance State, call a provider or ingest
the bundle. A protocol-consistent violated Observation, Challenge findings or partial Run remains a
valid execution account rather than a command failure.

Challenge Plans will select exact targets through stable traceability, never through paths, line
numbers, globs or silent whole-suite fallback. Mutation testing, broad static analysis, flakiness
repetition and qualification-oriented fault injection normally act as Challengers. Fault injection
with a direct recovery or durability oracle may instead implement a Check.

Generating a plan from repository authority, resolving its current decision targets, provider
capability discovery, native selector translation, execution and report import remain dependent
adapter and challenge-planning changes. The `plan`, `execute`, `import` and `ingest` Run operations
remain absent. Durable ingestion, authorization, retention and Subject-specific Assurance State
belong to the future ledger.

The optional Assurance Service remains isolated on its D42 v1 wire until the Run-ledger replacement
is accepted. It does not ingest Run bundles, and there is no compatibility bridge or Assurance
Service export command.

## Branches and rollout

Azimuth does not prescribe Git topology. One change may use several branches or repositories, and
one branch may carry several accepted changes, provided authority and revision accounts remain
explicit.

Archiving accepts a codebase transition; it does not assert universal production exposure. Deploy
immutable artifacts through the team's delivery system. Incidents, live measurements and rollout
results may motivate a later change, but they do not silently rewrite the archived account.
