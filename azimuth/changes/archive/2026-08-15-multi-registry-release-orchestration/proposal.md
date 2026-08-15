# Change: multi-registry-release-orchestration

Status: accepted and complete

Exploration: canonical-alpha-release
Carries decisions: CAR5, CAR6, CAR7, CAR8, CAR9, CAR10, CAR11, CAR12, CAR13

## Problem

The repository qualifies package contents and assurance image buildability, but it has no release
pipeline that produces the complete selected artifact population from one tag. There is no retained
candidate account, no cross-platform disposable-consumer smoke evidence, no checksum or provenance
account and no rule for resuming after one immutable registry succeeds and another fails.

The private-deployment change also exposed a concrete orchestration defect. Pull request run
`31818896569` passed the complete root gate in 40 minutes 57 seconds, while the identical
merge-triggered run `31822229595` was killed by GitHub's 45-minute job limit after 45 minutes 34
seconds. Both ordinary CI events compile the two-image, two-platform matrix from a cold BuildKit
cache. Repeating release-only ARM64 compilation in the ordinary repository job gives neither a
usable release candidate nor a dependable feedback bound.

## Outcome

Ordinary CI continues to execute one canonical repository check, but that check no longer owns the
release-only multi-platform image matrix. One separate release-rehearsal workflow qualifies package,
native-binary and image lanes independently, retains their exact outputs, assembles one complete
tagged candidate account and proves disposable consumption on every selected platform. BuildKit
cache is shared by release-rehearsal executions without weakening the exact output account.

Every retained downloadable candidate has a SHA-256 checksum and GitHub build-provenance subject.
Image candidates retain their selected platform account and acquire registry provenance when
published. Publication is tag-bound and target-aware: an absent target may be published, an exact
existing target is preserved, and a conflicting immutable target fails closed. Completion requires
post-publication retrieval of all seven registry identities, all three native binaries and both
two-platform image indexes.

## Scope

In scope:

- one fast canonical repository workflow and one independently bounded release workflow;
- independent package, native-binary and assurance-image qualification lanes;
- release-only Linux AMD64 and ARM64 image builds with reusable BuildKit cache;
- retained Cargo, NuGet, npm, native archive and OCI candidates from one source tag;
- disposable consumers for the five packages, three native binaries and both image indexes;
- a deterministic candidate manifest with exact subjects, sizes and SHA-256 checksums;
- GitHub build-provenance attestations for downloadable binaries and container images;
- tag, version, source revision and catalog agreement before publication;
- target-aware publication planning across crates.io, NuGet, npm, GitHub Releases and GHCR;
- preservation of exact existing immutable targets and rejection of conflicting targets;
- resumption of only missing targets after partial publication; and
- post-publication retrieval before release completion.

Out of scope:

- creating or rotating registry credentials;
- claiming the `@azimuth` npm scope;
- publishing `v0.1.0-alpha.1` during this change;
- changing the selected artifact identities, version or platforms;
- SBOM generation, cross-ecosystem signing or stable compatibility guarantees;
- application authentication or internet-facing assurance hardening; and
- Drim consumption or referrals.

## Affected claims

Add `framework/release-orchestration#qualification-lanes-converge` at standard criticality. A lane
failure must remain local while the complete account fails closed, because one global job already
exceeded its hosted limit and supplied no retained release output.

Add `framework/release-orchestration#tagged-candidates-are-verifiable` at critical criticality.
Substituting a candidate after qualification would make checksums and provenance describe a
different executable than the public alpha and would affect every consumer of the immutable
version.

Add `framework/release-orchestration#qualified-candidates-compose` at standard criticality. A
candidate that packs but cannot be installed or started blocks dogfood before affecting durable
consumer data.

Add `framework/release-orchestration#partial-publication-resumes-safely` at critical criticality.
Registry versions are immutable; overwriting, ignoring or misclassifying a conflicting existing
target would create a release identity whose components no longer share one source account.

## Completion conditions

- The ordinary `check` workflow still has one canonical checkout and one `./scripts/check.sh`
  command, but the command does not build the release-only image matrix. A successful fresh hosted
  run completes below the existing 45-minute job limit.
- A separate rehearsal derives every package, native target, image and image platform from
  `release/artifacts.json`; a missing, duplicate or unexpected candidate fails the complete account.
- Package, binary and image lanes can fail independently and retain successful immutable outputs
  without declaring the release complete.
- Both image candidates build for Linux AMD64 and ARM64 in the release lane, use a reusable
  repository-scoped BuildKit cache and are not rebuilt by the ordinary root job.
- Disposable consumers install and exercise the public entry point of every Cargo, NuGet and npm
  package, execute each native CLI on its selected runner and start both images on every selected
  platform available to the rehearsal.
- One manifest binds the catalog version and tag, full source revision, every retained filename,
  byte size, SHA-256 checksum, registry identity and platform population.
- Synthetic changes to a candidate, checksum, tag, revision, identity, platform or population fail
  before publication planning.
- The workflow requests GitHub build-provenance attestations for every downloadable candidate and
  published image digest, with permissions bounded to the release jobs.
- A dry publication ledger demonstrates absent, exact-existing and conflicting-existing states for
  every registry kind. Exact existing targets are never overwritten; only absent targets are
  selected on resume; conflicts fail closed.
- Release completion is impossible until post-publication retrieval revalidates all selected
  packages, three native archives and both multi-platform image indexes against the retained
  account.
- The complete repository gate and a clean hosted release rehearsal pass without publishing or
  authenticating to a public registry.
