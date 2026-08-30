# Release qualification and publication

`.github/workflows/ci.yml` is the reusable build-and-qualification graph. Pull requests run it
against GitHub's synthetic merge revision, pushes to `main` run it against the resulting repository
revision, and `.github/workflows/publish.yml` calls it once more at an annotated version tag. The
same source, Assurance, package, native, image-platform, deployment, image-assembly and release
qualification lanes therefore guard every context.

Each job retains immutable workflow artifacts for downstream jobs in the same run. Package
qualification consumes the package job's archives instead of rebuilding them. Deployment imports
the exact amd64 image fragments with Compose builds disabled. Multi-platform image assembly consumes
the same platform fragments. BuildKit caches are scoped by image and architecture, but caches never
establish artifact identity.

The tag-triggered workflow publishes only after the reusable graph succeeds and the protected
`release` environment is approved. It derives the expected artifact population directly from
`release/artifacts.json`, rejects missing, duplicate and unexpected files, and writes
`SHA256SUMS`. No manifest or promotion record crosses workflow runs.

## Hosted evidence

Use `gh` to inspect hosted runs:

```sh
gh run list --workflow "continuous integration" --branch BRANCH --limit 5
gh run view RUN_ID --json conclusion,event,headSha,jobs,url
gh run download RUN_ID --dir DOWNLOAD_DIRECTORY
```

For `.azimuth/release/ordinary-workflow-receipt.json`, record the pull request head revision as
`sourceRevision`, the successful run's revision as `executionRevision`, its URL and duration, and
the current hashes of `.github/workflows/ci.yml` and `scripts/check.sh`.

For `.azimuth/release/release-workflow-receipt.json`, use the successful tag run. Derive `jobs` and
`subjects` from `release/artifacts.json`; record the SHA-256 of the run's `SHA256SUMS` as
`artifactSetSha256`. Confirm every exact subject's GitHub attestation names the tag revision before
recording `attestedSubjects`. Hash `.github/workflows/ci.yml`, `release/orchestrate.py` and
`release/candidates.py` for their named receipt fields. A running workflow cannot honestly attest
its own future conclusion, so qualification may defer these hosted receipts until the run completes.

Validate refreshed records with Azimuth's release qualification commands. A failed run is diagnostic
information only and cannot replace a successful receipt.

## Publishing a version

Before tagging, update every native manifest and lockfile, `release/artifacts.json`,
`release/acceptance.py`, the resource manifest, migration inventory, contracts, accepted model and
adopter/operator documentation to one version. Review the bundled skills as consumer workflows;
`.agents/skills/` is development input and never a release source.

Configure the protected `release` environment with `CARGO_REGISTRY_TOKEN`, `NUGET_API_KEY` and
`NPM_TOKEN`. The npm identity must administer the `@azimuth-sh` organization. The workflow's bounded
GitHub token owns GitHub Release and GHCR writes.

Create and push one annotated tag at the intended `main` revision:

```sh
git tag --annotate v0.1.0-alpha.6 --message "Azimuth 0.1.0-alpha.6"
git push origin v0.1.0-alpha.6
gh run list --workflow release --branch v0.1.0-alpha.6 --limit 1
gh run watch RUN_ID --exit-status
```

The workflow verifies that the annotated tag matches the catalog and names a commit in `main`
history. After the same-run build succeeds, publication reads all public targets, rejects immutable
conflicts, publishes only absent targets and normalizes required npm distribution tags. A rerun on
the same tag preserves exact public targets and resumes absent ones. Completion retrieves every
package, native archive and image publicly, validates provenance and platforms, and uploads a
`public-release-completion` artifact.

If a transient failure leaves the tagged source and release bytes unchanged, rerun the workflow. If
the correction changes source or artifacts, keep the failed tag immutable, advance the version and
create a new tag. Any partially published version is permanently consumed.
