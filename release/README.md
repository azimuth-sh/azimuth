# Release evidence

The rehearsal workflow produces candidates and GitHub attestations but does not publish them. Its
tracked receipts are external evidence records, not files that local code can derive without
observing GitHub. Refresh a receipt only after the changed executable inputs are committed and the
corresponding pull-request workflow succeeds.

Use `gh` for every hosted observation:

```sh
gh run list --workflow "release rehearsal" --branch "$(git branch --show-current)" \
  --event pull_request --limit 5
gh run watch RUN_ID
gh run view RUN_ID --json conclusion,headSha,jobs,url
gh pr view --json headRefOid
gh run download RUN_ID --dir DOWNLOAD_DIRECTORY
```

For `.azimuth/release/release-workflow-receipt.json`, record the pull request's `headRefOid` as
`sourceRevision` and confirm that it equals the run's `headSha`. Record the downloaded candidate
account's `revision` as `executionRevision`, then confirm that revision against every signed
attestation. Derive `jobs` and `subjects` from `release/artifacts.json`; hash the downloaded
`candidates.json` for `candidateAccountSha256`. Hash `.github/workflows/release.yml`,
`release/orchestrate.py` and `release/candidates.py` for their named SHA-256 fields. Query each
downloaded subject digest with `gh api repos/drim-dev/azimuth/attestations/sha256:DIGEST`, decode the
signed statement and record `attestedSubjects` only after every exact subject names the workflow
and execution revision.

For `.azimuth/release/ordinary-workflow-receipt.json`, record the same revision identities, compute
the successful `check` job duration from its `startedAt` and `completedAt` values, and hash
`.github/workflows/ci.yml` plus `scripts/check.sh`. Copy each accepted receipt into the active
change before archiving it; if the change is already archived, retain the corrected historical
copy beside its outcome. Then validate the records with:

```sh
./release/check.sh --experiments-executed
```

A failed run is diagnostic evidence only. It cannot replace either successful receipt.

## Public alpha publication

`.github/workflows/publish.yml` is owner-dispatched and accepts one successful rehearsal run id. It
downloads that run's retained candidates and account; it does not build candidates. A dry run reads
all public targets, derives the absent/exact/conflicting plan and records credential readiness with
zero writes.

Configure the `release` environment with `CARGO_REGISTRY_TOKEN`, `NUGET_API_KEY` and `NPM_TOKEN`.
The npm identity must administer the `@azimuth` organization. The workflow's bounded GitHub token
owns GitHub Release and GHCR access. NuGet and GHCR expose no non-writing probe that proves the
right to create an unused identity, so the preflight records that limitation explicitly.

After the publication change is merged, run the release rehearsal on `main`, wait for its complete
account, create the annotated catalog tag at that same main revision and push the tag. Use `gh` to
dispatch the no-write preflight against the tag:

```sh
gh workflow run publish.yml --ref v0.1.0-alpha.1 \
  -f rehearsal_run_id=RUN_ID -f dry_run=true
gh run watch PUBLICATION_RUN_ID --exit-status
```

Inspect the retained `publication-preflight` artifact before dispatching with `dry_run=false`. The
write run re-reads every registry, rederives the plan and publishes only its absent set. A rerun
after interruption preserves exact public targets. The workflow emits `public-release-completion`
only after post-publication retrieval validates all ten targets, GitHub Release support assets,
GHCR platform sets and provenance against the retained account.
