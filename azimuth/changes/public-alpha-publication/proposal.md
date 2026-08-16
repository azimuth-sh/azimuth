# Change: public-alpha-publication

Status: approved

Exploration: canonical-alpha-release
Revision: anonymous-public-image-retrieval
Carries decisions: CAR5, CAR11, CAR12, CAR13, IPI1, IPI2, IPI3, IPI4; completes change-map node F

Intent delta: none
Because: the unavailable `@azimuth` npm organization and the domain-independence boundary require
replacement of accepted identity values, but this Azimuth version does not project replacement
operations (D21.4, D24). The alpha has no consumers, so the current scenario is revised in place
under the repository's no-backward-compatibility policy and the limitation is recorded in the plan.

## Problem

The retained-candidate workflow proves that ten selected subjects compose, but it deliberately
stops before public registry access. The repository therefore has no adapter that converts fresh
crates.io, NuGet, npm, GitHub Releases and GHCR reads into planner state, no credential preflight
covering all five publication boundaries and no owner-triggered operation that publishes the exact
retained bytes. The synthetic completion account cannot establish that `v0.1.0-alpha.1` exists.

Live reads on 2026-08-15 found the five package identities absent, but the bare `@azimuth` npm
organization was unavailable. The user created `@azimuth-sh`, acquired `azimuth.sh` and transferred
the release repository to `azimuth-sh/azimuth`. The local npm credential cannot yet read the new
organization, the local environment has no crates.io token, and the repository has no publication
secrets. Starting writes in that state would make a partial release likely before recovery has
been exercised against real adapters.

The first write-enabled attempt on 2026-08-15 falsified the assumption that every credential can
be authorized through a non-mutating request. A crates.io token restricted to `publish-new` cannot
call the legacy `/me` endpoint, and the repository read made with a bounded GitHub Actions token did
not report its job-declared write permissions. The attempt stopped with zero writes. Requiring
broader tokens would hide the provider boundary rather than strengthen the release account.

The repaired write-enabled attempt on 2026-08-15 falsified the assumption that npm infers a
non-`latest` distribution tag from a prerelease version. It published six targets that a later
preflight retrieved as exact, then npm rejected the first tarball before writing it. Two NuGet
targets and two npm targets were absent from the public read account two minutes later. That
observation does not distinguish a rejected NuGet write from indexing delay, so recovery waits for
a later read rather than treating immediate absence as proof that no write occurred.

The later 2026-08-15 observation retrieved both NuGet versions but classified their raw archive
checksums as conflicts. NuGet's official verifier reports the same content hash before and after
ingestion and verifies the downloaded repository signatures; unpacked comparison found the added
`.signature.p7s` entry was the only content difference. This falsifies raw archive equality as the
NuGet identity rule. The revised adapter requires a valid repository signature and equality of all
non-signature paths and payloads.

GitHub accepted a targeted rerun of the skipped image-provenance job but skipped it again because
the original publication dependency remained failed. The tag cannot move after six immutable
targets exist: doing so would break the candidate account and its attestations. Recovery therefore
runs reviewed orchestration from a later revision while keeping candidate authority at the
unchanged tag. The public account records both revisions and accepts image provenance only through
the retained archive, its deterministic index digest and the publication-revision attestation.

The repaired no-write run 31905158474 retrieved eight exact targets and selected only the two npm
packages. Write run 31905266399 published those tarballs, attached both image attestations and
emitted a ten-target completion receipt. An independent npm read then found both `alpha` and
`latest` assigned to `0.1.0-alpha.1`. This falsified the assumption that `npm publish --tag alpha`
alone prevents a first package version from becoming `latest`, and it falsified the completion gate
because that gate did not read mutable distribution tags. Hosted normalization run 31907022845 was
then rejected before its first deletion, and an interactive WebAuthn-authorized deletion returned
HTTP 400. The npm registry package contract requires `latest`; these second observations falsify
the proposed removal, not the immutable release. The revised planner records stable versions and
accepts this alias only while none exist.

Write-enabled run 31937065763 then passed the corrected npm rule and emitted a ten-target receipt,
but the completion job had authenticated to GHCR before reading both images. Independent anonymous
reads returned HTTP 403 because both container packages retained their default private visibility.
This falsified authenticated retrieval as evidence of public availability. After the owner made
both packages public, anonymous reads returned the retained index digests and selected platform
sets. The checker is revised in place because public retrieval was already the stated predicate;
it now makes the absence of registry credentials observable.

## Outcome

One explicit repository-owner workflow consumes a successful rehearsal run whose source revision
equals the annotated release tag. It downloads and verifies the ten retained candidates, retrieves
the complete public target population, rejects conflicts, checks every credential to the strongest
non-mutating extent its provider permits and publishes only targets classified absent by the
accepted planner. An unprobeable write authorization remains unknown until its first write and is
recorded as such rather than represented as authenticated.

The operation may be rerun after interruption. Exact public targets are preserved, conflicting
immutable targets stop the operation before another write, and completion is emitted only after a
fresh retrieval validates all packages, three GitHub Release archives, both multi-platform GHCR
indexes, checksums and provenance against the tag-bound retained account. npm distribution tags
are retrieved separately from tarball identity; prerelease completion requires the derived channel
to select the version and forbids the same version at `latest` after a stable version exists.

## Scope

In scope:

- provider adapters for crates.io, NuGet, npm, GitHub Releases and GHCR public state;
- a non-publishing preflight that checks tag, rehearsal revision, target state, credential presence
  and provider-supported identity before any registry write;
- an owner-dispatched GitHub Actions workflow that downloads rather than rebuilds retained
  candidates;
- publication of the Rust crate, two NuGet packages, two npm packages, three native archives,
  `SHA256SUMS`, the candidate account and two multi-platform assurance images;
- replacement of the unavailable npm scope and Drim-owned repository and image coordinates with
  the controlled `azimuth-sh` owner and `azimuth.sh` homepage;
- GitHub prerelease metadata and provenance for published GHCR index digests;
- exact-existing preservation, conflict rejection and missing-target resumption through the
  accepted planner;
- repair of mutable npm distribution tags without republishing exact tarballs; and
- a public completion receipt derived from post-publication retrieval.

Out of scope:

- changing `0.1.0-alpha.1`, `v0.1.0-alpha.1`, the artifact population or qualified platforms;
- creating additional registry accounts or generating credentials;
- rebuilding a candidate during publication;
- SBOMs, non-GitHub signing, additional platforms or compatibility promises;
- Drim consumption, referrals or any other domain dependency; and
- application authentication or internet-facing assurance deployment.

## Affected claims

The change modifies
`framework/release-artifacts#registry-identities-match-contract` and realizes or refreshes evidence
for
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
- Every required credential is present and every provider-supported non-mutating identity check
  succeeds before writes. npm administration is observed directly; crates.io `publish-new`, NuGet
  push, GitHub Release and GHCR write authorization remain explicit first-write limitations.
- Every retained package names `https://github.com/azimuth-sh/azimuth` as its source and
  `https://azimuth.sh` as its homepage; every image names both through OCI labels.
- The owner-triggered workflow downloads the successful rehearsal artifacts by run id and never
  invokes a candidate build.
- A repair revision can execute against an explicit candidate tag without changing candidate
  authority; the receipt distinguishes the candidate and publication revisions.
- Each registry adapter receives only planner-selected absent targets. Exact targets are preserved
  and any conflicting immutable identity prevents a publication plan.
- A failed operation retains successful public targets and a rerun selects only the remaining
  absent targets.
- npm prerelease publication names its prerelease channel explicitly; completion accepts npm's
  required `latest` alias only while the package has no stable version.
- NuGet retrieval accepts the provider-added repository signature only when the official verifier
  accepts it and every non-signature payload remains equal to the retained candidate.
- GitHub Releases exposes all three native archives, `SHA256SUMS` and `candidates.json` under the
  annotated prerelease tag.
- Each GHCR identity exposes one anonymously retrievable index containing Linux AMD64 and ARM64
  and has GitHub provenance bound to its published digest.
- Fresh public retrieval after publication passes the accepted completion checker for all ten
  subjects and records an immutable completion receipt.
- Unit tests, a no-write hosted preflight, the complete repository gate and the actual publication
  workflow pass without reading Drim or another domain checkout.
