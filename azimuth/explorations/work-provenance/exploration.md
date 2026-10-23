# Exploration: Work Provenance ledger

Id: work-provenance
Created: 2026-10-23
Status: approved

## Objective

Retain a durable, append-only ledger of every completed workflow attempt that shaped a Requirement's definition, implementation and verification: which action was performed with which outcome, by which execution instance and what kind of agent (model, effort, harness and guidance, each with its basis), when, and what it used and produced. Each Requirement's history is computed from that ledger and shown beside its real fate in the Azimuth Assurance Service (AAS), with optional authored Failure Attributions on top.

The purpose is a dataset from which conclusions about model and harness quality can be drawn, and a trail for investigating why a Requirement failed.

## Boundaries

- **Non-assessment invariant.** Work Provenance is a separate domain. Nothing in it is ever an input to Requirement Review currentness, Requirement Status, acceptance or any other assessment.
- **No fingerprint participation.** Provenance never enters model, Requirement, review or workflow attempt identity (agent-tier F15).
- **Not telemetry.** Tokens, cost, transcripts, tool calls and traces are out of scope. An attempt carries at most an optional opaque `trace` reference.
- **Not accountability.** Whether an agent may hold an accountable identity (agent-tier open question 1) and what independence suffices (agent-tier F6) are not decided here. `actor` keeps its current independence semantics.
- **No new deployable.** The domain is hosted in AAS with its own schema and API. Core stays dependency-free and network-free.
- **No normalized cross-provider scales.** Effort is retained provider-native; comparison across providers is an analysis concern.
- **Non-normative.** This exploration authorizes no change; each candidate goes through `azimuth-propose`.
- **Relationship to `agent-tier-representation`.** This exploration resolves its E10 (hold the provenance record pending measurement) and is the instrument for its E2 and E13 measurements. It inherits F14, F15, F21 and E13 as constraints. Agent quality, which that exploration put out of scope, is in scope here.

## Existing context

- `contracts/workflow.md` and `tools/azimuth/src/workflow_state.rs` define workflow results; `actor` is a harness assertion supplied with `--actor`.
- `contracts/assurance-reviews.md:64` defines optional review `provenance {agent, model, reassesses}`.
- `docs/intended-architecture.md:32`, `:124`, `:132` and `:136` place worker state and agent traces outside AAS and anticipate Resolution records.
- The active `workflow-coordination` change owns the workflow contract and engine and is not archived.
- The only real workflow instance inspected is drim-dev's archived `2026-10-22-private-api-service-ingress` change: 48 results in `.azimuth/workflow.json` plus `workflow-before-*` snapshots.

## Findings

### Facts

- **F1 — A workflow result records who only as a free label.** It retains `task {action, target}`, `basis`, `actor`, `outcome`, output `records` and `inputs` (`tools/azimuth/src/workflow_state.rs:2185`). It has no model, effort, harness or time. `actor` is compared by equality for reviewer independence (`:1818`, `:1845`, `:1895`).
- **F2 — Real actor labels conflate instance, role, attempt and implicitly model.** The drim-dev instance uses `claude`, `claude-author`, `claude-implementer`, `claude-coordinator`, `rev-proposal-2`, `independent-spec-reviewer-subagent`, `req-reviewer-caller-authority-4` and `dima`.
- **F3 — Requirement Reviews carry partial provenance.** Optional `provenance {agent, model, reassesses}` and `reviewed_at_ms`. In practice `Model: claude-opus-5-5` is recorded and `Agent:` holds a subagent path such as `/root/hosted_check_review`, a role rather than a harness. There is no effort. AAS stores these beside transport facts (`submitted_by`, `submitted_at`, `provenance_basis`).
- **F4 — Workflow history is not durable.** drim-dev `.gitignore:127` ignores `azimuth/changes/**/.azimuth/`; the 48-result history exists only on one machine, and earlier generations survive only as manual `workflow-before-*` snapshots. AAS stores no workflow history.
- **F5 — Most tasks are linked to a module, not a Requirement.** Only `review-requirement` targets name a Requirement. Proposal, spec, design and verification tasks target the change or a module; planning and implementation target the change.
- **F6 — Correction loops are already observable.** The history contains author → rejected review → corrected author → accepted review, which is the signal agent-tier E13 requires.
- **F7 — Derived prose assigns model provenance to harness telemetry.** `docs/intended-architecture.md:136`: "Harness telemetry supplies model and instruction provenance, duration, cost, retries and human effort … without making AAS the repository of every agent trace." `:132` keeps unfinished attempts out of AAS. This prose holds the least authority.
- **F8 — AAS already holds the fate side and the lineage.** Requirement Status per deployment, Check Results, Verification Challenge findings, reviews, and change → candidate model revision → main-model publication.
- **F9 — Core is dependency-free and network-free.** `tools/azimuth/Cargo.toml` declares no dependencies; transport lives in scripts such as `scripts/publish-assurance.py`.

### Inferences

- **I1 — The data is process provenance, not telemetry.** In W3C PROV terms an attempt is an Activity performed by an Agent that used and generated Entities. It is low-volume, durable, linked to entities and read long after the fact. F7 groups it with telemetry; the boundary F7 actually protects is "not every agent trace", which this data is not.
- **I2 — Hosting risk is semantic leakage, not storage location.** A separate domain with an explicit non-assessment invariant addresses it without a separate deployable.
- **I3 — Fingerprint-closure diffs are a better attribution key than declared deltas.** Comparing consecutive candidate revisions surfaces every change that altered a Requirement's definition or its dependencies, including drift its deltas never mention.
- **I4 — Without guidance fingerprints, a model effect cannot be distinguished from a skill-version effect.**
- **I5 — Self-reported or orchestrator-transcribed values record what was requested, not what ran.** The dataset is trustworthy only where each value's source is visible.
- **I6 — Dropping rejected and superseded attempts builds survivorship bias into the dataset and makes E13 unmeasurable.**
- **I7 — Computed fate supports only correlation.** Causal conclusions need authored attribution.

## Decisions

### D1 — A new exploration that overturns agent-tier E10

Agent-tier puts quality out of scope, and this initiative is mostly about quality. Its F14, F15, F21 and E13 are inherited rather than bypassed, and it receives a note that E10 is resolved here.

### D2 — Work Provenance is its own domain, hosted in AAS, kept as a ledger never used for assessment

The data is provenance (I1); AAS already holds the fate side and the change → revision lineage (F8); the leakage risk is handled by the domain boundary (I2). Records are authored locally, committed with the change, and transported at publication, mirroring Requirement Reviews, so workflow stays offline-capable.

### D3 — A structured agent descriptor with a closed field set

`kind` (`model | human`), `model` (provider-canonical id, e.g. `claude-opus-5-5`), `effort` (provider-native, verbatim), `harness {name, version}` and `guidance` (fingerprints of the skills and instructions actually supplied). The same descriptor replaces the free `agent`/`model` labels in review provenance. Canonical values make attempts comparable; guidance fingerprints separate model effects from skill effects (I4).

### D4 — `actor` is narrowed to the execution instance

`actor` names one execution, a session or subagent run, and encodes no role, attempt or model. It remains the independence key with unchanged rules. Kind moves to the descriptor; accountability stays with agent-tier. Deriving independence from the descriptor would forbid same-model fresh-context review and silently settle agent-tier F6. For humans, instance and accountable identity coincide.

### D5 — Requirement attribution is computed from fingerprint-closure changes

A Requirement's history consists of the changes whose candidate revision changed its fingerprint closure (definition, Checks, bindings, realizing source), each labeled `intent` or `dependency`. Each attempt within a change keeps its honest scope: `requirement`, `module` or `change`. Nothing is authored, so nothing can be misreported (I3). Edits outside the workflow register no candidate and appear as unexplained drift, which is itself a signal.

### D6 — Every descriptor field records its basis

Basis is `harness | declared | unknown`. Harness-supplied values are preferred; declared values are permitted and marked; `unknown` is never replaced by a guess. Analysis can filter to attested rows (I5).

### D7 — The ledger holds every completed attempt

Failed, rejected and superseded attempts, restarted instances with an explicit predecessor link, and abandoned changes with an `abandoned` disposition are all retained (I6). An abandoned change has no candidate revision and so attaches to no Requirement under D5, but it still counts in model statistics. An attempt that never submitted a result is telemetry and leaves no ledger record.

### D8 — Fate is computed; causes are optionally attributed

Fate comes from AAS facts: the Requirement Status timeline per deployment, violated Check Results, Verification Challenge findings and rejected Requirement Reviews. An optional, append-only Failure Attribution record in the provenance domain names an adverse fact, the attempts believed to have **caused** it or **allowed it through**, a rationale, a confidence and the investigator's descriptor. A later attribution may revise an earlier one. Optional, so attributions are written when someone actually investigates rather than to clear a gate (agent-tier F9.1) (I7).

### Accepted defaults

- Provenance records are transported under the existing publisher credential with `provenance_basis: publisher-transported-assertion`; no new credential role.
- The tool stamps `handoff_captured_at` when `workflow next` produces the handoff and `recorded_at` on update; duration is derived. Agent-supplied times are `declared`.
- Descriptor, time and attribution are excluded from every fingerprint, including attempt identity, so adding provenance never changes a replayed result's identity.
- The `private-api-service-ingress` instance may be backfilled with every descriptor field `unknown` and actors as recorded.
- An attempt may carry an optional opaque `trace` reference.
- New work builds on `workflow-coordination` rather than amending it in place.

## Rejected alternatives

- **Fold into `agent-tier-representation`** — its objective is accountable identity, and it excludes quality.
- **Leave E10 standing and proceed** — contradicts an active decision by omission.
- **Repository ledger as the only home, analyzed offline** — loses the status-with-history view and makes every investigation a hand-built join across systems.
- **A separate Provenance service** — a second deployable, authentication surface and cross-service join for about 50 records per change. Revisit only if consumers other than Azimuth appear.
- **A non-assessment record class inside the existing assurance domain** — blurs the boundary the invariant depends on.
- **Classify as harness telemetry** — it is lineage linked to entities, not operational measurement.
- **Free-text or model-only descriptors** — not comparable, and cannot separate effort or guidance effects.
- **A normalized effort scale across providers** — builds a false equivalence into the ledger.
- **Drop `actor`, or key independence on the descriptor** — renames the same concept or forbids same-model review.
- **Attribute by declared deltas only** — misses dependency drift, the failure case that matters most.
- **Agent-authored per-task Requirement attribution** — self-reported, unverifiable, and lets the subject shape its own record.
- **Harness-attested values only** — discards human work and harnesses without integration.
- **Only final instances, or only successful attempts** — survivorship bias.
- **Mandatory Failure Attribution** — ceremony and rubber-stamped causes.

## Residual risks

- **R1 — Leakage through people.** The invariant stops the tool from using provenance in assessment; it cannot stop a reviewer from relaxing because a strong model passed first time. Showing history beside status makes that temptation visible.
- **R2 — Effort may not be attestable.** If harnesses do not expose it, effort is mostly `declared` or `unknown` and conclusions about effort stay weak. Depends on X1.
- **R3 — Sample size and confounding.** One owner, few projects and uncontrolled task difficulty support anecdotes, not statistical claims. The ledger records facts; validity of conclusions is an analysis responsibility.
- **R4 — Attribution quality.** Agent-authored Failure Attributions are themselves unmeasured outputs. The investigator descriptor makes that traceable, not correct.
- **R5 — Coverage gap.** Changes that bypass the workflow register no candidate revision and leave unexplained drift in Requirement histories.
- **R6 — Personal data.** The ledger names humans and records when they worked. Acceptable for a single team; multi-tenant hosting needs a retention and visibility policy.
- **R7 — Unfinished dependency.** `workflow-coordination` is active and unarchived; its contract may still shift.

## Open questions

- **U1** — Should Failure Attribution later merge with the Resolution records anticipated by `docs/intended-architecture.md:124`? Left open until either has a schema.
- **U2** — The committed ledger's exact shape and location in the change folder: a projection of `workflow.json`, or the instance with its predecessor generations. A design question for C3.
- **U3** — Once agent-tier open question 1 is decided, does the descriptor need an accountable-principal field, or do `actor` and approval cover it? Left to agent-tier.
- **U4** — Analysis surfaces beyond `report provenance` JSONL and the Requirement page, such as cross-project views or model comparison. Deferred until real data exists.
- **U5** — Retention and visibility of human provenance under multi-tenant hosting (R6).

### Experiments

- **X1 — Probe each harness for attestable fields.** For Claude Code and Codex, determine which of model, effort and harness version a hook or wrapper can read from actual session metadata rather than configuration or self-report, and whether a subagent's actual model is distinguishable from the requested one. Decides how much of the descriptor can carry `harness` basis, and whether R2 holds.
- **X2 — Backfill `private-api-service-ingress`.** Test whether restart snapshots link unambiguously as predecessor generations, and whether the computed fingerprint-closure history for its two Requirements matches what actually happened. The first real test of D5 and D7.

## Result

Shared understanding was reached on 2026-10-23. Eight decisions and their defaults are recorded; U1–U5 are deliberately left open, and X1 and X2 are identified. Seven candidate changes are mapped in `change-map.md`; C1 with C3 is the minimal slice that makes history durable and enriched without service work. No change is created or approved by this exploration.
