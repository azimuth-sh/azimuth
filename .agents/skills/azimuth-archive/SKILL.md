---
name: azimuth-archive
description: >-
  Accept, finalize and archive a completed Azimuth change after current facets, engineering checks,
  outcomes and rollout-dependent conditions are satisfied. Use at the local or federated
  completion boundary; never manufacture execution facts or commit implicitly.
---

# Finalize and archive a change

Archive records an accepted semantic transition. It is not a branch merge or deployment command.

## Preconditions

1. Read completion conditions and verify every plan and work-package item is complete.
2. Confirm intent deltas are applied and current design names only mechanisms that exist.
3. Confirm every current framework Claim remains routine and has no inapplicable Check, Evidence
   Binding or Qualification.
4. Run required builds, ordinary tests and composed gates. Emit fresh manifests and run
   `azimuth validate` over their union.
5. Inspect `azimuth report traceability` when relationships changed. Confirm version 2 export for
   public graph changes.
6. When Run bundles changed, use `azimuth run verify --bundle <file>...` over every relevant
   correction set and `azimuth run inspect --bundle <file>...` for its deterministic account.
   Apply the strict [Run bundle format](../../../azimuth/formats/run-bundle.md) without treating a
   valid negative or partial fact as a protocol failure.
7. Write `outcome.md` with `Status: accepted`, `## Departures` and `## Residual decisions`.
8. Set the proposal to `Status: accepted and complete` only after the preceding facts hold.

## Local acceptance

```text
azimuth change finalize <id> [model and manifest options]
azimuth change archive <id> --date YYYY-MM-DD [model and manifest options]
```

Finalization fingerprints the accepted model. Archive must fail if that fingerprint is stale.

## Federated acceptance

Repository-local archive is not project acceptance. Retain the complete accepted-active workset,
make the content-preserving archive commit in the singular authority repository, execute composed
engineering checks over the post-archive revision tuple, then run:

```text
azimuth project accept-change --project <catalog> --before <active-workset> \
  --after <archive-workset> --change <id> --date YYYY-MM-DD --out <snapshot>
```

The CLI verifies both immutable accounts. It creates no Git commit, deployment, Run or provider
receipt. `azimuth run verify` and `azimuth run inspect` likewise perform no plan generation,
provider execution, report import, durable ingest or Assurance State projection. The D42 service
wire remains isolated, and no Assurance Service export command exists.
