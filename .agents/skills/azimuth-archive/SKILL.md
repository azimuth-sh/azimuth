---
name: azimuth-archive
description: Accept, finalize, and archive a completed Azimuth change after current facets, evidence, judgments, outcomes, and rollout-dependent conditions are satisfied. Use at the completion boundary for local or federated changes; never manufacture evidence or commit source-control changes implicitly.
---

# Finalize and archive a change

Archive records an accepted semantic transition. It is not a branch merge or a deployment command.

## Preconditions

1. Read the proposal's completion conditions and verify every plan and work-package item is complete.
2. Confirm intent deltas are applied, current design names only existing mechanisms, and lasting
   verification deviations are current.
3. Run every required build, test, manual-evidence import, detector test and composed check. Run
   `azimuth check` over fresh manifests.
4. Invoke `azimuth-verify` for every new or stale critical judgment.
5. Write `outcome.md` with `Status: accepted`, `## Departures` and `## Residual decisions`.
   Framework experiments may record Measurements; production changes do not owe that section.
6. Set the proposal to `Status: accepted and complete` only after the preceding facts hold.

## Local acceptance

Run:

```text
azimuth change finalize <id> [model and manifest options]
azimuth change archive <id> --date YYYY-MM-DD [model and manifest options]
```

Finalization fingerprints the accepted model. Archive must fail if that fingerprint is stale.

## Federated acceptance

Repository-local archive is not project acceptance. Retain the complete accepted-active workset,
make the content-preserving archive commit in the singular authority repository, execute composed
evidence over the post-archive revision tuple, then run:

```text
azimuth project accept-change --project <catalog> --before <active-workset> \
  --after <archive-workset> --change <id> --date YYYY-MM-DD --out <snapshot>
```

The CLI verifies both immutable accounts. It does not create Git commits, deployments or execution
receipts. Report those external actions separately.
