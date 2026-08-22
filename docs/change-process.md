# Change delivery in the evidence control plane

Status: **operating guidance**. The implementation, its tests and the parser contracts under
`contracts/` remain authoritative.

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
boundary or still has unresolved product choices. `azimuth explore create|list|show` scaffolds and
lists the package under `azimuth/explorations/`. Persist only shared decisions and a bounded change
map. Research does not silently become current framework authority.

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
`work-packages.md`. The parser requires a parsable `Status`, a non-empty `Objective` and non-empty
`Owns` paths that are relative and never escape the checkout, and it detects overlapping ownership
and dependency cycles. `Depends on` and `Evidence` are read but not required, so declaring them is
a working convention rather than an enforced one. Workers do not finalize, archive or edit shared
change state.

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

A non-routine change uses the current decision graph:

```text
Check -> Evidence Binding -> Qualification
Claim composition -> Claim Judgment
Challenger -> Challenge Plan -> exact Qualification or Claim Judgment
```

A Check has one atomic terminal proposition. Each binding relates that Check to exactly one
case-level Claim and owns form, exact required context, challenge domain and policy. The binding id
is also its sole Qualification id. One Check may bind to several Claims and one Claim may receive
several Checks.

Each standard or critical case Claim also has one total-composition Claim Judgment. Evidence
Bindings and Judgments name one Decision Policy whose open forms must be covered by current
Challengers and Plans. The project Challenge Schedule assigns every required or declared form
exactly once to `gate | scheduled`. These are repository decisions and declarations, never Run
results.

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
As a working practice, exercise an enumerated surface against its real enumerator and a temporary
untagged negative member before acceptance; no artifact enforces that rehearsal.

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

## Run and adapter execution plane

Azimuth implements a standalone [`azimuth-run-bundle` version 1](../contracts/run-bundle.md)
exchange. A Run binds one exact Subject and may contain Check executions, Challenger executions or
both. Its plan and actual semantic selection are explicit, and its retries and work units reduce to
one Observation per actually selected Check and one Challenge Result per selected Challenger
target. Challengers search for objections to Qualifications or Claim Judgments; a clean result is
not positive product evidence.

Use the service-free protocol commands to verify or inspect an already normalized bundle or
correction set:

```text
azimuth run verify --bundle <file> [--bundle <file> ...]
azimuth run inspect --bundle <file> [--bundle <file> ...] [--format text|json] [--out <file>]
```

These commands validate the protocol account only. They do not establish that repository
fingerprints remain current, apply execution results to Assurance State, call a provider or ingest
the bundle. A protocol-consistent violated Observation, Challenge findings or partial Run remains a
valid execution account rather than a command failure.

Repository Challenge Plans select exact current qualified Qualification and current accepted Claim
Judgment targets through stable traceability, never through paths, line numbers, globs or silent
whole-suite fallback. The seven selector forms preserve every selected, missing, stale, rejected,
invalid, inapplicable or unresolved candidate. Mutation testing, broad static analysis, flakiness
repetition and qualification-oriented fault injection normally act as Challengers. Fault injection
with a direct recovery or durability oracle may instead implement a Check.

Configure provider work explicitly in strict `azimuth/adapters.json`, then use:

```text
azimuth adapter verify [--config <file>]
azimuth run plan --request <file> [--model <dir>] [--standards <file>] \
  [--workspace <file>] [--manifest <file>...] [--config <file>] [--out <file>]
azimuth run execute --plan <file> [--predecessor <bundle>...] \
  [--config <file>] [--out <file>]
azimuth run import --plan <file> --input <id>=<file>... \
  [--predecessor <bundle>...] [--config <file>] [--out <file>]
```

Core loads the complete unselected model and accepts Check-only, Challenge-only and mixed strict
requests. Every request selection names one explicit configured capability and finite work units;
each Challenge request also names an authored Plan and nonzero candidate cap. Planning resolves the
fixed requested Plan union, fails on any adverse candidate, requires exact Qualification context
and covers every selected decision's policy-required forms. It has no `--only` or partial-model
path and never adds a capability or widens a selector.

Each Challenge selection records its schedule lane and exact semantic scope. Each launch route
projects every source-backed scope item to one accountable input with the same kind, id and
fingerprint plus its locator account. Scope changes semantic Plan identity; locators change launch
identity. The adapter translates this frozen account rather than loading the Azimuth model.

Execute and import stage executable, resource and input content from the streams core hashes,
clear the child environment and bound both output streams. On supported hosts, the adapter starts
in a fresh process group before its code runs. One deadline bounds core request writing, response
and diagnostic reads and process wait. Core signals remaining group members on every terminal path.
An adapter descendant can deliberately use `setsid`, `setpgid` or an equivalent to leave the group;
it cannot extend core's wait beyond the deadline, but its termination is not guaranteed. This is
not non-escapable descendant containment, daemon supervision, hostile-code isolation or a
filesystem or network sandbox.

Core validates the strict response, exact route provenance, actual selection and complete bundle
before atomic publication. Repeatable predecessors must form one linear correction chain; the
response is revision zero or the exact next complete revision.

A valid violated Observation, Challenge finding, partial or cancelled Run, or adapter-returned
protocol-valid `timed-out` Run fact is an execution fact and exits zero. A host-enforced process
deadline is a transport timeout and exits one, as does a semantic, identity, content, other
transport or bundle mismatch. The returned `timed-out` fact is valid only when its complete bundle
arrives within the host deadline; the host timeout publishes nothing. CLI and schema failures exit
two. Neither nonzero class leaves an output file.

Challenge Results are exactly `clean | findings | inconclusive`. Clean is only a negative search
fact and creates no credibility or product evidence. Every planned Challenge omitted from a
partial, cancelled or timed-out Run has one exact execution diagnostic and no fabricated Result;
omitting scheduled work is allowed deferral, while gate omission records execution failure. Added
or substituted targets, context, units or scope are selection mismatches and publish nothing.

`model.extract` execution is absent. Durable ingestion, authorization, retention and
Subject-specific Assurance State belong to the future Run ledger. Current planning defines no
cache validity, cadence, historical-applicability or cross-Subject reuse semantics. Adapters are
bounded short-lived processes; there is no daemon, webhook, inbound gateway or long-running adapter
boundary.

The optional Assurance Service remains isolated on its alpha 1 wire until the Run-ledger replacement
is accepted. It does not ingest Run bundles, and there is no compatibility bridge or Assurance
Service export command.

## Branches and rollout

Azimuth does not prescribe Git topology. One change may use several branches or repositories, and
one branch may carry several accepted changes, provided authority and revision accounts remain
explicit.

Archiving accepts a codebase transition; it does not assert universal production exposure. Deploy
immutable artifacts through the team's delivery system. Incidents, live measurements and rollout
results may motivate a later change, but they do not silently rewrite the archived account.
