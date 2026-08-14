# Change delivery and assurance

Status: **operating guidance**. This document composes decisions already made about changes,
evidence and optional ownership into one end-to-end protocol. It is not a parser contract, a Git
workflow or a deployment system. `azimuth/changes/README.md`, the three format contracts and
`docs/decisions.md` remain authoritative where this guide disagrees with them.

The example is an automatic-refunds feature because it requires all three evidence strengths:
storage constraints can make duplicate refunds unrepresentable, executions can demonstrate the
user and provider paths, and production detectors can reveal settlement that stops making
progress. The same sequence applies to another domain; the named claims and evidence do not.

---

## Accountabilities, not job boundaries

The three facets create three loci of accountability (D3, D3.1). They do not require three job
titles or grant any role exclusive authorship of an artifact.

| Accountability | Decides | Typical capability |
|---|---|---|
| Intent owner | what must be true and how much it matters | analysis or product |
| Mechanism owner | what makes it true and how strongly | software development |
| Evidence owner | what would justify belief and how freshness is maintained | quality engineering |

An SRE may operate an alert whose adequacy is owned by the evidence owner. A developer may
implement a component test whose required scope and oracle were chosen by the evidence owner. A
quality engineer may direct an agent to write that test and a developer may review its concurrency
and fixture mechanics. Ownership answers who must notice insufficiency; it does not answer who is
allowed to type code.

An agent is an authoring and review instrument, never a fourth accountability. Whoever accepts its
output must be able to inspect the result at the level the delegated work requires.

---

## Example claims

The intent owner first separates consequences that would otherwise be hidden inside one feature:

| Claim | Criticality | Consequence |
|---|---|---|
| total refunded never exceeds total captured | critical | financial loss |
| one refund identity is applied at most once | critical | duplicate provider operation |
| an accepted refund eventually becomes completed or failed | critical | unresolved money |
| the receipt explains pending, completed and failed states | standard | user confusion |
| a malformed provider event does not block later events | standard | stalled processing |
| the receipt heading changes to `Refund` | routine | presentation only |

The routine claim stops at intent. It may have an ordinary test, but it acquires no Azimuth tags,
mechanism declaration or judgment merely because critical claims exist in the same change (D20).

---

## Sequence

### 1. Propose the transition

The intent owner writes the problem, scope, affected claim ids, exclusions and completion
conditions in `azimuth/changes/<id>/proposal.md`. Completion conditions distinguish engineering
acceptance from rollout goals. “Alert rules exist and their detectors fire under injected
violations” is an engineering condition. “Enable the feature for every user” is normally a
rollout condition and belongs to the delivery system.

If a production observation really is needed to accept the solution, the proposal says so before
implementation. Examples include a provider with no representative sandbox or a latency predicate
that cannot be sampled under representative pre-production load.

### 2. Write and classify intent

The intent owner authors the intent delta. The mechanism and evidence owners challenge ambiguous
predicates, missing failure states and criticality, but neither silently changes the business
meaning while designing a solution or a test.

Criticality is classified by consequence, not implementation complexity. A one-line authorization
guard can be critical; a multi-component preference can be routine.

### 3. Decide the mechanism

The mechanism owner writes `design.md` only when alternatives, boundaries or failure modes make a
decision worth preserving. For refunds this might compare a synchronous provider call with a
transactional intent, outbox and worker, then name the selected uniqueness, authorization,
idempotency and delivery mechanisms.

The evidence owner participates early because an unobservable or uninjectable mechanism cannot be
rescued by downstream test cases. The design includes the telemetry and failure seams required to
observe it, while `verification.md` owns the claim that those observations are sufficient.

### 4. Plan evidence before it exists

The evidence owner records only deviations from the project standard, non-test evidence and
residual risk (D4.5). The plan is not a test inventory.

For the example it may require:

- generated or model-based demonstrations of the captured/refunded arithmetic;
- concurrent component evidence through the real HTTP boundary and database;
- provider decline, timeout and retry evidence through a controllable provider harness;
- one composed receipt journey;
- a manual charter for comprehension, keyboard use, screen-reader output and non-colour state;
- oldest-intent-age, dead-letter, worker-heartbeat and reconciliation detectors;
- detector tests using an injected clock, planted imbalance and synthetic alert time series.

A monitor is detection evidence: it says the team should learn about a violation, not that the
property holds. It cannot silently replace a demonstration floor (D4.1, D4.3).

### 5. Split work without splitting meaning

`plan.md` orders dependencies and bounded work packages. Several developers or agents may work in
parallel after shared contracts are frozen: persistence and provider authority, broker delivery,
receipt composition, detector configuration and evidence harnesses are plausible packages.

Each package states what it owns, what it consumes and what it produces. The change remains one
semantic transition even when its work uses several branches, merge requests or repositories.

### 6. Implement the mechanism and evidence

The mechanism owner implements production behaviour, telemetry and testability seams. Evidence
implementation is allocated by capability:

| Evidence form | Normal division |
|---|---|
| unit evidence | developer authors; evidence owner challenges cases |
| component and contract evidence | shared; developer owns harness, evidence owner owns adequacy |
| e2e evidence | evidence owner often leads; developers keep service boundaries testable |
| manual charter | evidence owner authors and executes or delegates to a named specialist |
| metrics and rules | developer or SRE implements; evidence owner judges detector adequacy |
| detector test | shared; evidence owner defines failure, implementer builds the detector |

Automated test code is software and receives ordinary engineering review. A passing generated test
is not accepted until somebody can say which plausible wrong implementation it rejects.

### 7. Establish pre-production evidence

CI extracts linkage and runs the machine tier. It can find missing facets, insufficient forms,
unresolved bindings, absent detector tests and stale or failed receipts. It cannot decide whether
the predicate is true or the test discriminates.

An immutable release candidate is then identified by source commit, artifact digest, configuration
and schema version. Manual work is performed against that candidate, never against an unnamed
“current staging.”

A manual charter is still not evidence. The linked execution record names the candidate,
environment, executor and observations. Its imported receipt carries the provider, external case
and run, URL, instant, outcome, evidence form, expiry and source fingerprint. Only a passed,
unexpired imported receipt contributes demonstration evidence (D23).

### 8. Judge the complete account

Import manual receipts before the final judgment; a new receipt is a new fingerprint input and
correctly makes an older judgment stale. For every required critical judgment, the judge reads the
claim, realization sites, mechanism bindings, automated evidence, relevant manual record,
operational rules and detector tests.

The judge asks whether each site establishes part of the predicate, whether each evidence item
rejects a plausible wrong implementation, whether its declared form is honest and whether the spec
omits behaviour a reader would need. The judgment can withdraw trust; it cannot supply evidence
that is absent (D18, D28).

### 9. Accept and archive the codebase transition

Accepted intent is applied to package `spec.md` files; mechanisms that actually exist are distilled
into sibling `design.md`; lasting evidence deviations and residuals enter sibling
`verification.md`; departures and framework-evaluation observations enter `outcome.md`. The change
is finalized and archived only after the accepted current model is hole-free.

Archiving records semantic acceptance of the codebase, not universal production exposure. The
archive remains immutable if production later teaches the team something new; a fix, rollback or
changed claim is another change.

For a federated project, integration retains the complete accepted-active workset, commits the
archive-only move in the singular authority repository, reruns composed evidence for that new tuple
and passes both worksets to `azimuth project accept-change`. The command verifies completion,
singular authority, unchanged archive content, no unrelated revision movement and no source/model
edits in the archive commit before it emits the post-archive project snapshot. Azimuth does not make
the Git commit or mint the external execution receipt.

The optional `Measurements` section used by this repository measures Azimuth itself. It is not a
field for ordinary production changes. Product and operational metrics belong to their product and
delivery systems unless a result changes an accepted assurance decision.

### 10. Roll out the accepted artifact

The normal deployment source is an immutable artifact built from a protected mainline or the
team's established release-candidate commit. The same artifact is promoted through staging,
limited production exposure and wider rollout. A mutable developer branch is suitable for a
preview environment, not production.

Feature flags, configuration or traffic routing select the initial population. Before exposure,
the delivery owner confirms metrics are scraped, rules evaluate, a notification-path test reaches
its destination, dashboards have a baseline and an on-call owner has a response procedure.

If limited exposure succeeds, rollout expands. If it fails, the flag is disabled or the artifact
is rolled back and a corrective change records the new knowledge. The archived change is not
rewritten.

### 11. Keep evidence alive

Archival ends authoring of the transition, not assurance of its claims:

- test evidence is re-established by CI;
- a manual receipt expires at its declared boundary;
- editing an examined implementation, evidence site, plan or binding stales its judgment;
- a removed detector or detector test becomes a machine hole;
- alert and reconciliation outcomes feed incidents and later changes.

The current machine tier establishes repository-owned rules, bindings and synthetic detector
behaviour. It does not yet establish that a live metrics backend is scraping or that notification
routing is healthy. A mature integration can import expiring operational receipts for rule
evaluation, scrape health, dead-man heartbeats, notification-path tests or canary results without
making the core vendor-specific.

---

## Branches and changes

The default relationship is many-to-many:

- one small change commonly fits one short-lived branch and merge request;
- one large change may use several work-package branches and repositories;
- one release branch may carry several already accepted changes.

The invariants are that every branch refers to the semantic change it contributes to, mainline
remains safe to build, and production receives a reviewed reproducible artifact. Azimuth does not
gain truth by prescribing Git topology.

Long-lived integration branches are a local exception, not framework guidance. They delay the
machine and agent feedback whose freshness model assumes small, inspectable transitions.

---

## When production observation gates acceptance

The default is archive before limited production exposure. Reverse the order only when the
proposal states that a production observation is necessary evidence and names its acceptance
window, oracle, artifact identity and failure action.

In that exceptional flow the implementation may already be integrated safely behind a disabled
flag while the change remains `implemented, rollout acceptance pending`. The exact mainline
artifact is exposed to the limited population. A successful observation supplies the final receipt
and stales any judgment that did not inspect it; finalization and archive follow a refreshed
judgment. Failure disables exposure and keeps the change active or archives it as rejected.

If active-but-applied changes become common, this separation has failed: the archive has become a
release tracker rather than a semantic record. That observation would falsify the default boundary
and require explicit rollout state in the change model rather than more prose.
