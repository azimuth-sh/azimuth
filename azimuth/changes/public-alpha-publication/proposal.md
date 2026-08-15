# Change: public-alpha-publication

Status: approved

Exploration: canonical-alpha-release
Carries decisions: CAR5, CAR8, CAR11, CAR12, CAR13; completes change-map node F

Intent delta: none
Because: the accepted release-orchestration intent already defines public completion; this change
binds real registry adapters and executes that existing obligation without changing its predicate.

## Problem

The retained-candidate workflow proves that ten selected subjects compose, but it deliberately
stops before public registry access. The repository therefore has no adapter that converts fresh
crates.io, NuGet, npm, GitHub Releases and GHCR reads into planner state, no credential preflight
covering all five publication boundaries and no owner-triggered operation that publishes the exact
retained bytes. The synthetic completion account cannot establish that `v0.1.0-alpha.1` exists.

Live reads on 2026-08-15 found the five package identities absent. The authenticated npm identity
`mitro52` cannot administer the `@azimuth` organization, the local environment has no crates.io
token, and the repository has no publication secrets. Starting writes in that state would make a
partial release likely before recovery has been exercised against real adapters.

## Outcome

One explicit repository-owner workflow consumes a successful rehearsal run whose source revision
equals the annotated release tag. It downloads and verifies the ten retained candidates, retrieves
the complete public target population, rejects conflicts, proves every credential before its first
write and publishes only targets classified absent by the accepted planner.

The operation may be rerun after interruption. Exact public targets are preserved, conflicting
immutable targets stop the operation before another write, and completion is emitted only after a
fresh retrieval validates all packages, three GitHub Release archives, both multi-platform GHCR
indexes, checksums and provenance against the tag-bound retained account.

## Scope

In scope:

- provider adapters for crates.io, NuGet, npm, GitHub Releases and GHCR public state;
- a non-publishing preflight that checks tag, rehearsal revision, target state and bounded
  credentials before any registry write;
- an owner-dispatched GitHub Actions workflow that downloads rather than rebuilds retained
  candidates;
- publication of the Rust crate, two NuGet packages, two npm packages, three native archives,
  `SHA256SUMS`, the candidate account and two multi-platform assurance images;
- GitHub prerelease metadata and provenance for published GHCR index digests;
- exact-existing preservation, conflict rejection and missing-target resumption through the
  accepted planner; and
- a public completion receipt derived from post-publication retrieval.

Out of scope:

- changing `0.1.0-alpha.1`, `v0.1.0-alpha.1`, selected identities or qualified platforms;
- creating registry accounts, claiming the `@azimuth` npm organization or generating credentials;
- rebuilding a candidate during publication;
- SBOMs, non-GitHub signing, additional platforms or compatibility promises;
- Drim consumption, referrals or any other domain dependency; and
- application authentication or internet-facing assurance deployment.

## Affected claims

No accepted intent changes. The change realizes and refreshes evidence for
`framework/release-orchestration#tagged-candidates-are-verifiable`,
`framework/release-orchestration#qualified-candidates-compose` and
`framework/release-orchestration#partial-publication-resumes-safely`. It discharges the accepted
registry-credential, public-retrieval and GHCR-digest-provenance residuals only from observed public
results; a missing credential or registry target remains a named incomplete condition.

## Completion conditions

- Preflight reads every selected public identity and emits one closed-world planner state before
  any publish command can execute.
- Preflight fails before writes unless the annotated tag, successful rehearsal source revision,
  retained account revision, catalog version and all ten candidate digests agree.
- Every required credential is checked against only its target registry. Missing `@azimuth`
  administration, crates.io access, NuGet push access or GitHub package/release permission prevents
  publication.
- The owner-triggered workflow downloads the successful rehearsal artifacts by run id and never
  invokes a candidate build.
- Each registry adapter receives only planner-selected absent targets. Exact targets are preserved
  and any conflicting immutable identity prevents a publication plan.
- A failed operation retains successful public targets and a rerun selects only the remaining
  absent targets.
- GitHub Releases exposes all three native archives, `SHA256SUMS` and `candidates.json` under the
  annotated prerelease tag.
- Each GHCR identity exposes one index containing Linux AMD64 and ARM64 and has GitHub provenance
  bound to its published digest.
- Fresh public retrieval after publication passes the accepted completion checker for all ten
  subjects and records an immutable completion receipt.
- Unit tests, a no-write hosted preflight, the complete repository gate and the actual publication
  workflow pass without reading Drim or another domain checkout.
