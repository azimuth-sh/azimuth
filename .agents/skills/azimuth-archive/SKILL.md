---
name: azimuth-archive
description: Accept, finalize and archive a completed Azimuth change after current facets, engineering checks, outcomes and rollout-dependent conditions are satisfied. Use at the local or federated completion boundary; never manufacture execution facts or commit implicitly.
---

# Finalize and archive a change

Archive records an accepted semantic transition. It is not a branch merge or deployment command.

## Preconditions

1. Read completion conditions and verify every plan and work-package item is complete.
2. Confirm intent deltas are applied and current design names only mechanisms that exist.
3. Confirm every current framework Claim remains routine and has no inapplicable Check, Evidence Binding, Qualification or Claim Judgment. For any authorized non-routine consumer Claim, confirm its strict Claim Judgment, Decision Policy required forms and one project Challenge Schedule are current before accepting planning behavior.
4. Run required builds, ordinary tests and composed gates. Emit fresh manifests and run `azimuth validate` over their union.
5. Inspect `azimuth report traceability` when relationships changed. Confirm version 4 export for public graph changes.
6. When Run bundles changed, use `azimuth run verify --bundle <file>...` over every relevant correction set and `azimuth run inspect --bundle <file>...` for its deterministic account. Apply the strict [Run bundle format](../../../contracts/run-bundle.md) without treating a valid negative or partial fact as a protocol failure.
7. When adapters changed, run `azimuth adapter verify`, create applicable Check-only, Challenge-only and mixed launch plans and exercise execute or import through the strict [adapter](../../../contracts/adapter.md) and [launch-plan](../../../contracts/run-launch-plan.md) contracts. Confirm process-limit, process-group, schema, identity and bundle failures publish no output, while valid adverse facts remain successful exchanges. Confirm Challenge planning resolves every candidate disposition, current accepted decision, required form, schedule lane, semantic scope and accountable launch input from the complete model, with one explicit capability and no automatic widening. Confirm an allowed incomplete scheduled omission has one exact `challenge-selection` diagnostic and no fabricated result. Confirm one bounded core exchange whose deadline covers request writing, stream draining and core's wait. Confirm core establishes the process group before spawn, signals it on every terminal path and cleans members and inherited pipes only while they retain group membership. Authorized descendants may escape with `setsid`, `setpgid` or equivalent, and their termination is not guaranteed. This is not non-escapable descendant containment, a sandbox, daemon supervision or hostile-code isolation. Adapters remain short-lived, not daemons, webhook hosts or long-running supervisors.
8. When mechanism extractors changed, prove that the existing two-argument marker produces one ecosystem-semantic qualified site, exact path-free typed binding and companion Artifact. Reject ambiguity, unsupported identity, non-normal/outside-root locators and any file-path fallback.
9. Write `outcome.md` with `Status: accepted`, `## Departures` and `## Residual decisions`.
10. Set the proposal to `Status: accepted and complete` only after the preceding facts hold.

## Local acceptance

```text
azimuth change finalize <id> [model and manifest options]
azimuth change archive <id> --date YYYY-MM-DD [model and manifest options]
```

Finalization fingerprints the accepted model. Archive must fail if that fingerprint is stale.

## Federated acceptance

Repository-local archive is not project acceptance. Retain the complete accepted-active workset, make the content-preserving archive commit in the singular authority repository, execute composed engineering checks over the post-archive revision tuple, then run:

```text
azimuth project accept-change --project <catalog> --before <active-workset> \
  --after <archive-workset> --change <id> --date YYYY-MM-DD --out <snapshot>
```

The CLI verifies both immutable accounts. It creates no Git commit, deployment, Run or provider receipt. `azimuth run verify` and `azimuth run inspect` likewise perform no planning, provider execution, report import, durable ingest or Assurance State projection. `azimuth run plan` resolves Check-only, Challenge-only or mixed requests from the complete model; it does not execute `model.extract`, ingest Runs, derive Assurance State, infer cache validity or authorize historical reuse. A clean Challenge Result is not product evidence, and `deferred` is not a result. The legacy service API and version 2 project-snapshot wire remain isolated, and no Assurance Service export command exists.
