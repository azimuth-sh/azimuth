# Change delivery in the evidence control plane

Status: **operating guidance**. The implementation, its tests and the parser contracts under `contracts/` remain authoritative.

Azimuth separates repository-owned intent and reviewed meaning from execution facts. During the fast-moving alpha no current framework Requirement declares `Realization:` or `Verification:`. Changes therefore use ordinary engineering tests and release gates without enrolling those tests as Azimuth evidence.

## Accountabilities

| Accountability | Decides |
|---|---|
| Intent owner | what must be true and which reviews it owes |
| Realization owner | what makes it true in the code and which boundaries matter |
| Verification owner | what would justify belief when the Requirement declares `Verification:` |

These are accountabilities, not job titles. An agent is an authoring and review instrument; the person accepting its output remains accountable for the result.

## Current alpha sequence

Before entering a stage, a consumer agent follows the matching installed skill and asks the running CLI for parser-sensitive details with `azimuth reference show <artifact>`. The target repository and its instruction hierarchy are the complete working context; ordinary consumer work does not consult the canonical Azimuth checkout. Templates scaffold an artifact, references define its current grammar, and skills own the workflow.

### 1. Explore uncertainty

Use an exploration before commitment when a topic spans several changes, crosses an unfamiliar boundary or still has unresolved product choices. `azimuth explore create|list|show` scaffolds and reads the active package under `azimuth/explorations/`. Persist only shared decisions and a bounded change map. Research does not silently become current framework authority.

Approval and archival are separate actions. After the actual exploration files are approved, leave the package active until a later explicit request invokes `azimuth explore archive <id> --date <YYYY-MM-DD>`. Archival requires the approved status and a complete disposition account, moves the package content-preservingly to the dated archive, and does not wait for identified changes or experiments to finish.

### 2. Propose one transition

Create one change id with singular authority. State the problem, outcome, in-scope and out-of-scope work, affected Requirement ids and completion conditions. Carry explicit exploration decisions when applicable.

New or changed framework Requirements declare no review until a later accepted change deliberately adds `Realization: reviewed` or `Verification: reviewed` after the codebase stabilizes. Review declarations follow consequence rather than implementation size, but this repository currently declares none to avoid manufacturing immature assurance decisions.

Add a change `design.md` only when alternatives, failure modes, migration order or boundaries need review. Add `work-packages.md` when work can be delegated safely. `plan.md` remains an implementation sequence for the change; it is not a verification artifact or test inventory. `azimuth change brief <change> --package <package>` renders one contextual delegation handoff; authoring grammar comes from `azimuth reference`, not from that brief.

### 3. Freeze shared contracts

Before parallel work, freeze public formats, identities and ownership boundaries. Validate `work-packages.md`. The parser requires a parsable `Status`, a non-empty `Objective` and non-empty `Owns` paths that are relative and never escape the checkout, and it detects overlapping ownership and dependency cycles. `Depends on` and `Evidence` are read but not required, so declaring them is a working convention rather than an enforced one. Workers do not finalize, archive or edit shared change state.

### 4. Implement observable behavior

Implement product behavior, structural mechanisms, telemetry and testability seams inside the
approved boundary. Add Realizes only where a production site genuinely establishes part of a
parent Requirement that declares `Realization:`. Other Requirements do not require source linkage.

Ordinary unit, component, integration and release tests remain normal software. A passing test is useful engineering feedback, but it is not an Azimuth Check merely because it executes in CI.

### 5. Update current facets

Keep intent deltas proposed while implementation is unfinished. During proposal review, run
`azimuth change intent-capture` for replacements or removals, then
`azimuth change intent-preview --out <file>` and inspect the target spec and diff. Near acceptance,
refresh the preview and review any drift. When implemented behavior satisfies the reviewed intent,
run `azimuth change intent-apply --preview <file>` to update package `spec.md` mechanically.
`finalize` and `archive` only verify that application occurred; they do not apply intent.
Distil only mechanisms that now exist into the module's `realization.md` when it helps the Realization Review; it is optional. Because no current Requirement declares `Verification:`, do not create package `verification.md`
files or reviews for this alpha transition.

A change that adds or keeps a Requirement declaring `Verification:` authors Checks and inline Check–Scenario contributions in `verification.md`, and enables any required packages explicitly. Surface field declarations, independent Expectation Sets and Probe origins describe module-specific verification without duplicating package mechanics. Source tags identify actual implementations. Emitters produce typed descriptors; `azimuth surface verify` checks runtime enumeration/expectation completeness and schema against those exact producers.

Review the full candidate account before relying on it. Prepare exact packets with `azimuth assurance review-input`; independently author and compile a Verification Review for each Requirement that declares `Verification:` (Checks, bindings, oracle independence, coverage, controls and the adequacy of declared or absent Verification Challenges) and a Realization Review for each that declares `Realization:` (whether the code at the realization sites does what the Scenarios say, consistent with `realization.md` where present). Neither requires an execution Verification Target. Rejected or stale reviews return work to correction and independent reassessment.

Execute Checks and machine-executed Verification Challenges against exact source, artifact or deployment Verification Targets. Preserve actual measurement timestamps; importing an old report does not renew its freshness. A Verification Challenge targets the exact implementation and Check basis while retaining its own execution Verification Target. Current mutation integration captures exact batch source input and native results, not materialized per-mutant Verification Targets or product Check Results. Report `clean | findings | inconclusive` and resolve relevant objections; clean is not positive assurance.

Deployment Requirement Status is computed in AAS from applicable evidence for an observed deployment; there is no authored Requirement Status record. Its `supported | violated | unresolved` conclusion retains missing coverage and failed controls. Neither structural validation nor a successful execution writes an independent assessment, and Satisfied requires an accepted current Verification Review.

Export authority with `azimuth assurance authority`; register a candidate revision in the existing project without replacing the default model. Change acceptance follows required reviews and the human decision. Candidate registration and CI publication grant no deployment authorization.

### 6. Validate the repository account

Emit fresh language manifests and run:

```text
azimuth validate --manifest <manifest>...
azimuth report traceability
azimuth export --out model.json
```

Validation reports categorized Findings. Traceability is a derived, deterministic view over Scenarios
and stable graph relations. Export writes the current versioned model projection. None of these commands executes native
tests or creates an execution fact.

Run focused engineering tests while iterating, then affected component and composed suites. As a working practice, exercise an enumerated surface against its real enumerator and a temporary untagged negative member before acceptance; no artifact enforces that rehearsal.

### 7. Record the outcome

Complete plan and work-package statuses. Write `outcome.md` with departures and residual decisions. Framework experiments may record measurements of Azimuth itself; ordinary product changes do not owe that section.

### 8. Finalize and archive

Under the merge-first lifecycle the change is finalized and archived on its pull-request branch, and the merge to `main` is what updates the current model. Confirm completion conditions, current facet updates, fresh manifests and all required engineering checks against the model merged with current `main`. Run `azimuth change finalize`, then archive only after the finalization fingerprint is current. These commands do not create Git commits, provider executions or deployment records.

Conflicts between concurrent changes surface where Git surfaces them: when the branch is rebased onto `main`, the intent deltas and current facets are re-applied and the complete merged model is re-validated by finalization. The facet-level rejection in `azimuth release compose` is not the conflict detector.

A Requirement that declares a review is ratified by records committed in the change. Each owed review needs a sealed native `verification-review` or `realization-review` record under `reviews/` (with the packets required to recompile it), and each added Requirement that declares a review needs a human decision in `acceptance.json` (see `contracts/validation.md`). Finalization recomputes ratifications for every Requirement that declares a review: the latest record of each kind among the change reviews and the model's review heads is assessed against the merged model, and only an accepted current one ratifies. A Requirement left without a ratifying review keeps its `missing-requirement-assurance` or `missing-realization-review` Finding and fails finalization — so a review or head made stale by a rebase or by another change's edit to a blocking dependency needs re-review, recommit and finalization again. Finalization then writes each new ratifying review as the Requirement's review head in `verification-review.md` or `realization-review.md` beside the spec. The acceptance alone waives nothing. The finalization receipt lists the ratifying reviews as `ratifying_reviews` `{requirement, kind, review}` entries, and the archive moves `reviews/` and `acceptance.json` with the change, so the ratification is part of the decision record. The change record states what the decider relied on, and Azimuth never contacts the Azimuth Assurance Service itself.

For federation, retain complete accepted-active and tested-archive worksets. The singular authority repository owns the content-preserving archive move. `azimuth project accept-change` verifies the two complete accounts, applies the same ratification rule to every committed review and acceptance, and emits a snapshot; a local account cannot substitute for project acceptance.

## Run and adapter execution plane

Azimuth implements a standalone [`azimuth-run-bundle` version 2](../contracts/run-bundle.md)
exchange. A Run binds one exact Verification Target and may contain Check executions, Verification
Challenge executions or both. One Check execution may serve several selected Scenarios and reduces
independently to one Check Result per Scenario. Verification Challenges search the exact implementation and Check basis for weaknesses the Checks miss; a clean
result is not positive product evidence.

Use the service-free protocol commands to verify or inspect an already normalized bundle or correction set:

```text
azimuth run verify --bundle <file> [--bundle <file> ...]
azimuth run inspect --bundle <file> [--bundle <file> ...] [--format text|json] [--out <file>]
```

These commands validate the protocol account only. They do not establish that repository fingerprints remain current, apply execution results to Assurance State, call a provider or ingest the bundle. A protocol-consistent violated Check Result, Challenge findings or partial Run remains a valid execution account rather than a command failure.

Compact Verification Challenge declarations select Requirements and Mechanisms through stable semantic identities. Planning pins the exact implementation target and resolves tagged implementation locations into accountable scope. Mutation targets, analysis/build dependencies and exercised Checks are separate. Missing mappings or ambiguous scope must not silently expand to the entire repository.

Configure provider work explicitly in strict `azimuth/adapters.json`, then use:

```text
azimuth adapter verify [--config <file>]
azimuth run plan --request <file> [--model <dir>] \
  [--workspace <file>] [--manifest <file>...] [--config <file>] [--out <file>]
azimuth run execute --plan <file> [--predecessor <bundle>...] \
  [--config <file>] [--out <file>]
azimuth run import --plan <file> --input <id>=<file>... \
  [--predecessor <bundle>...] [--config <file>] [--out <file>]
```

Core loads the complete unselected model and accepts Check-only, Challenge-only and mixed strict requests. Requests name configured capabilities and finite work units; Challenge selection names a declared Verification Challenge and a prepared scope report. Core resolves the exact implementation target fingerprint and scope rather than asking agents to maintain duplicate file inventories.

Launch inputs preserve source-backed semantic items and accountable locators. The adapter translates this frozen account into tool parameters and records exclusions, ambiguity and tool-driven scope expansion; it does not reinterpret the Azimuth model.

Execute and import stage executable, resource and input content from the streams core hashes, clear the child environment and bound both output streams. On supported hosts, the adapter starts in a fresh process group before its code runs. One deadline bounds core request writing, response and diagnostic reads and process wait. Core signals remaining group members on every terminal path. An adapter descendant can deliberately use `setsid`, `setpgid` or an equivalent to leave the group; it cannot extend core's wait beyond the deadline, but its termination is not guaranteed. This is not non-escapable descendant containment, daemon supervision, hostile-code isolation or a filesystem or network sandbox.

Core validates the strict response, exact route provenance, actual selection and complete bundle before atomic publication. Repeatable predecessors must form one linear correction chain; the response is revision zero or the exact next complete revision.

A valid violated Check Result, Challenge finding, partial or cancelled Run, or adapter-returned protocol-valid `timed-out` Run fact is an execution fact and exits zero. A host-enforced process deadline is a transport timeout and exits one, as does a semantic, identity, content, other transport or bundle mismatch. The returned `timed-out` fact is valid only when its complete bundle arrives within the host deadline; the host timeout publishes nothing. CLI and schema failures exit two. Neither nonzero class leaves an output file.

Challenge Results are exactly `clean | findings | inconclusive`. Clean is only a negative search fact and creates no credibility or product evidence. Every planned Challenge omitted from a partial, cancelled or timed-out Run has one exact execution diagnostic and no fabricated Result; gate omission records execution failure. Added or substituted targets, Execution Context, units or scope are selection mismatches and publish nothing.

`model.extract` execution is absent. The current Run ledger provides project-role authorization, append-only Run/review histories, main-model publication and exact-Verification-Target compact Assurance State. Current planning defines no cache validity, cadence, historical-applicability or cross-target reuse semantics. Adapters are bounded short-lived processes; there is no daemon, webhook, inbound gateway or long-running adapter boundary.

The earlier Assurance Service remains isolated on its alpha 1 wire; it does not ingest current Run bundles and has no compatibility bridge. The separate current `run-ledger` accepts those bundles and compact independent review records. `azimuth assurance authority` exports the full model projection, not persisted runtime records.

## Upgrading a consumer repository

Upgrade the CLI and each adopted annotation or emitter pin through their owning package managers as one reviewed cohort. Then run `azimuth update --check`, inspect `azimuth update --dry-run`, resolve managed-file, alias or component diagnostics, and apply `azimuth update`. The command is offline and updates only CLI-owned resources and their installation account.

If the release changes accepted account format, inspect the bundled migration reference, create a content-addressed plan with `azimuth migrate plan --out <file>`, resolve all review-required findings in the user-owned account, replan, and apply the exact plan. Normal validation never accepts the old grammar. Installations without `azimuth/installation.json` are not adopted automatically; the two known pre-manifest repositories preserve project-owned policy, remove reviewed generated guidance and initialize afresh.

## Branches and rollout

Azimuth does not prescribe Git topology. One change may use several branches or repositories, and one branch may carry several accepted changes, provided authority and revision accounts remain explicit.

Archiving accepts a codebase transition; it does not assert universal production exposure. Deploy immutable artifacts through the team's delivery system. Incidents, live measurements and rollout results may motivate a later change, but they do not silently rewrite the archived account.

## Check locator transition

The stable entity identity change preserves project-wide Requirement, Scenario, Mechanism and Check IDs independently of Module membership and Scenario parents. Typed references and source markers use stable names directly. Use explicit declaration-backed mappings, preserve unresolved stale targets and never add aliases or rewrite historical artifacts. Fresh extraction and planning produce current identities and fingerprints; an old decision or Run is not silently current. Entity kind remains explicit for explorer and protocol consumers. The existing test prohibition applies until tests are explicitly requested.

## Explicit workflow coordination

`azimuth workflow init --workflow change-development --inputs <file>` selects a shipped declarative process and expands declared module targets; `next`, `update` and `verify` coordinate its finite concrete graph. Later validated same-project inventories instantiate assurance tasks without a coordinator-authored DAG. `workflow definition --workflow change-development` exposes the pinned reusable contract. Request, plan steps and workflow tasks are distinct. `next` is read-only and gives full handoffs in `eligible` without recommending an execution order; no `next`, `blocked`, `current` or `superseded` fields are returned. The harness selects work, controls cost and permissions, and launches agents. `update` records actual results conditionally and idempotently; it does not perform work or grant permissions. See [workflow.md](../contracts/workflow.md) and `azimuth reference show workflow`.

Independent delta reviews cover intent completeness, realization support and verification adequacy; they stay local to the change. Their exact inputs, reviewer and findings remain separate from artifact completion, structural validation and human approval. Adverse outcomes expose configured correction work and independent reassessment. After implementation, the Verification and Realization Reviews assess the complete candidate verification and realization. Requirement Status assesses observed behavior from applicable evidence for a deployment. A valid adverse record is preserved; it does not satisfy readiness. No separate Check Review is required.

Agents can perform all review work locally. CI/CD executes deterministic Checks and transports the current accountable delivery package to AAS, rather than uploading every draft or correction attempt. Include exact dependencies, necessary provenance and relevant adverse or unresolved facts. A CI producer token authorizes Runs only; publishing unchanged local review records requires the authorized reviewer credential matching their original reviewer. Transport must not fabricate a CI reviewer or silently refresh review time.

Subsequent hosted Runs and deployment Requirement Status follow authorized deployment. AAS preserves immutable published reviews and correction chains, exposes both review dimensions independently of Verification-Target-specific Requirement Status, and never joins different Verification Targets solely because model revisions match. Missing or stale assessments pause progression without requiring model calls in CI. Generated packets, compiled bundles, Runs and local scope reports belong in ignored change `.azimuth/`; authored definitions and aggregate review records remain in their owning source account. This increment does not automate deployment, federation acceptance, archive handoff or production scheduling.
