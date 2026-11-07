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

Publication holds no long-lived registry secret, apart from the temporary first-publish token below. The `publish` job exchanges its GitHub OIDC token
(`id-token: write`, protected `release` environment) for short-lived credentials before any target
is written; the workflow's bounded GitHub token owns GitHub Release and GHCR writes. Register these
trusted publishers, each matching the top-level caller workflow `publish.yml` and the `release`
environment (the repository owner is `azimuth-sh` and the repository is `azimuth`):

| Registry | Where | Policy |
| --- | --- | --- |
| nuget.org | organization `azimuth-sh` > Trusted Publishing | Repository Owner `azimuth-sh`, Repository `azimuth`, Workflow File `publish.yml`, Environment `release`, scope `Azimuth.*` allowing new packages and new versions |
| npm | each of `@azimuth-sh/annotations`, `@azimuth-sh/emit`, `@azimuth-sh/testing-playwright` > Settings > Trusted Publisher | GitHub Actions, organization `azimuth-sh`, repository `azimuth`, workflow `publish.yml`, environment `release`, allowing `npm publish` and `npm dist-tag` |
| crates.io | crate `azimuth` > Settings > Trusted Publishing | repository owner `azimuth-sh`, repository `azimuth`, workflow `publish.yml`, environment `release` |

The NuGet policy owner must own every `Azimuth.*` package. The repository variable `NUGET_USER` is
the nuget.org username (profile name) passed to `NuGet/login`; the action documents it only as
"Your NuGet account username" and the policy goes inactive if its creating member leaves the
organization, so use the username of the active `azimuth-sh` member who creates the policy. NuGet
documents no first publish of a package ID through a policy, so `Azimuth.Testing.Xunit`, which has
never been public, is pushed after every other target. npm and
crates.io accept a trusted publisher only on a package that already exists, so the first version of
a new package is published once by an owner outside this workflow, then its policy is registered. A
newly created npm policy must complete its first successful publish within two days or it expires
(https://docs.npmjs.com/trusted-publishers), so create npm policies just before tagging. Tick both
"Allow npm publish" and "Allow npm dist-tag" on each: dist-tag access is off by default and is how
the planner moves the first prerelease to `latest`. The `publish` job needs npm
11.21 or newer, which the workflow installs; npm publishes provenance without a flag.

### First publish of `@azimuth-sh/testing-playwright`

That package has never been published, and npm accepts a trusted publisher only on an existing
package. For its first version alone, `release/publication.py` reads the `release`-environment
secret `NPM_FIRST_PUBLISH_TESTING_PLAYWRIGHT`, a granular npm token with publish rights, bypassing
two-factor authentication for publishing, scoped as narrowly as npm allows and expiring within 7
days. It is used only when the registry reports the package as absent, and the run stops before
writing any target if the token is missing. Every other target uses trusted publishing.

After the first release publishes the package: delete the secret and the token on npm, then add the
package's trusted publisher from the table above. Later releases then take the OIDC path and the
secret is never read.

The NuGet and crates.io exchanges run, and are required to return credentials, before the planner
selects or writes anything, so a registry-side policy fault there publishes nothing and a rerun on
the same tag is safe. npm has no exchange apart from publishing, so npm targets are written first:
a missing npm policy then fails before any other target is written.

Create and push one annotated tag at the intended `main` revision:

```sh
git tag --annotate v0.1.0-alpha.10 --message "Azimuth 0.1.0-alpha.10"
git push origin v0.1.0-alpha.10
gh run list --workflow release --branch v0.1.0-alpha.10 --limit 1
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


## Public test integration candidates

The first-alpha package cohort also includes `Azimuth.Testing.Xunit` (NuGet, .NET 8+) and
`@azimuth-sh/testing-playwright` (npm). Their native manifests, packed metadata, allowlisted file sets and
release version are qualified with the existing artifact cohort. Package qualification installs
the actual xUnit archive through a private NuGet cache/source mapping and executes its report API;
it installs the actual npm tarball and runs a real synthetic Playwright test with the packaged
fixture/reporter. These smokes do not contact AAS or require a browser download. Canonical source
checks separately cover failures, retries, contexts and missing captures through real runners.

`execution-context`, `test-report`, `test-mapping`, `native-test-results`, `deployment-assessment`
and `deployment-status` have independent catalog protocol versions (`test-report` and
`native-test-results` 1, the others 2). They are not inferred from package version
`0.1.0-alpha.10`. Public source adapters under `packages/python` remain
commit-pinned source inputs rather than a newly invented PyPI artifact identity.

Registry publication is not required for consumer CI adoption: build CLI/harness artifacts once
from the same full canonical Git commit, then distribute those immutable workflow artifacts.
For NuGet use a commit-specific preview package version with a matching consumer version, or an
isolated `NUGET_PACKAGES` cache and source mapping to the local candidate feed. Reusing a fixed
package version with a shared cache can silently select bytes from a different source revision.
Record the full tooling Git revision and artifact digests; never resolve floating main as identity.
