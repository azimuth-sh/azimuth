# Verification: framework/release-orchestration

## Claim: ordinary-ci-excludes-release-only-matrix
Scope: e2e
Quantification: universal
Oracle: direct

The static guard inspects the complete ordinary workflow and root command relation. A fresh hosted
run must finish successfully below the configured 45-minute limit without invoking the selected
image matrix.

## Claim: selected-lanes-are-independent
Scope: component
Quantification: universal
Oracle: direct

The rehearsal derives all lane jobs from the catalog and mutation cases fail each lane separately.
Retained output identifiers must be disjoint and available to convergence without a rebuild.

## Claim: complete-account-needs-every-lane
Scope: component
Quantification: universal
Oracle: direct

The candidate manifest validator compares exact sets for packages, native targets, images and image
platforms. Missing and duplicate mutations range over every subject; an unexpected subject is an
independent failing mutation.

## Claim: tag-catalog-and-revision-agree
Scope: component
Quantification: universal
Oracle: direct

Pull-request rehearsal force-updates only its local temporary annotated tag over the tested
revision, so a fetched public tag cannot make the isolated review run float or fail.
Owner-dispatched rehearsal honors an existing fetched tag. Both paths reject independent changes
to tag, version and full revision before any candidate is accepted.

## Claim: retained-downloads-have-checksums
Scope: component
Quantification: universal
Oracle: direct

Every retained regular file is hashed after lane convergence. A byte mutation to every subject in
the catalog-derived population must make manifest verification fail.

## Claim: executable-subjects-have-provenance
Scope: e2e
Quantification: universal
Oracle: direct

The hosted rehearsal submits every downloadable subject to GitHub's provenance action and validates
the workflow's exact subject population. Publication run 31905266399 attached attestations to both
GHCR index digests. A repair revision does not replace candidate provenance: completion must join
retained-archive provenance at the tagged revision, deterministic index identity and
published-digest provenance at the recorded publication revision. Final run 31938723090 retained
both complete provenance chains in `public-release-completion.json` after anonymous retrieval.

## Claim: packed-packages-install
Scope: component
Quantification: universal
Oracle: direct

Disposable consumers range over all five package candidates and use only their native archive
installation path. Each invokes the declared annotation or emitter entry point.

## Claim: native-binaries-run
Scope: component
Quantification: universal
Oracle: direct

The three selected operating-system runners extract their retained archive, execute the CLI and
compare its reported version with the catalog.

## Claim: selected-image-platforms-start
Scope: e2e
Quantification: universal
Oracle: direct

The population is every catalog-selected image/platform pair. The rehearsal imports each platform
from the retained OCI archive and observes its declared startup entry point.

## Claim: exact-existing-target-is-preserved
Scope: component
Quantification: universal
Oracle: direct

Synthetic registry state ranges over the complete selected target population. Exact existing
candidates yield no publication action and retain their retrieved identities.

## Claim: absent-target-is-selected
Scope: component
Quantification: universal
Oracle: direct

For each selected target, a mutation marks only that target absent and requires the publication
plan to select exactly it.

## Claim: conflicting-target-fails
Scope: component
Quantification: universal
Oracle: direct

For each registry kind, a retrieved identity with different content prevents the planner from
emitting any publication set.

## Claim: completion-needs-public-retrieval
Scope: component
Quantification: universal
Oracle: direct

The completion checker ranges over the complete public population and rejects each missing or
mismatched package, native archive, image index, checksum, provenance subject or platform. npm
mutations also require every derived prerelease channel to select its version. Retrieval records
the stable-version population: `latest` may select the prerelease only when that population is
empty, because the registry package contract requires a `latest` tag. Once a stable version exists,
the same alias is drift and completion fails without guessing the owner's intended stable target.
Exact tarballs remain preserved while mutable channel metadata is considered separately. A passed
receipt remains rollout-dependent until the release operation retrieves real public targets.
Image retrieval invokes `skopeo --no-creds`, and the hosted completion job has no registry login.
The command-shape test and workflow mutation guard therefore reject an implementation that lets an
organization credential make a private GHCR image satisfy the public predicate.

Final run 31938723090 retrieved all ten real targets through the corrected adapters, including both
GHCR indexes without credentials. Its retained receipt records the exact candidate and publication
revisions, all target identities and digests, both selected image platforms, the complete
provenance population and the npm channel plus stable-version state.

## Residual: registry-credentials-are-not-rehearsed
Accepted: write authorization is provider-dependent; discharge from the publication operation

No credential is used during rehearsal. Publication cannot begin until every credential is present
and every provider-supported non-mutating identity check succeeds through the bounded release
environment. A crates.io `publish-new` token, NuGet push key and GitHub Release or GHCR workflow
token expose their write authorization only through the first write; the resumable planner retains
that limitation rather than requiring broader credentials.

All ten immutable targets are publicly retrievable. The first npm versions also acquired `latest`.
Hosted run 31907022845 could not remove the aliases with a bypass-2FA token, and an interactive
WebAuthn-authorized removal reached the registry but returned HTTP 400. The registry package model
requires at least the `latest` dist-tag, so the attempted repair falsified the prior oracle rather
than the public package state. That failure required a fresh operation to record the empty
stable-version populations and pass the corrected completion oracle.

Write-enabled run 31937065763 passed the npm rule and retained ten exact immutable targets, but its
GHCR reads followed a workflow registry login. Independent anonymous reads then returned HTTP 403
for both images, proving that the packages were private and falsifying that receipt as public
completion evidence. After the owner changed both packages to public, anonymous registry-protocol
reads returned index digests
`85c76fa563950b75dc3e5bece5e72618d322aedd9dad965d26dee4679bdac329` and
`25704694ebb7bebbff77832018ba90fb516d502c5795f78604d27e13b2a6a719`, each with Linux AMD64 and
ARM64. Those observations made a new hosted receipt from the anonymous checker necessary.

No-write run 31938652438 selected zero writes, preserved ten targets and observed the same two
anonymous image index digests. Final run 31938723090 preserved all ten again, emitted completion
from the anonymous adapter and retained empty stable-version populations plus both required npm
tags. The exact completion receipt is stored with the active change; its SHA-256 is
`6c03769f4ade8709d7356a1629a4fce9617135a3254b8b9724446cdab7eda0ce`.

## Residual: provenance-is-not-a-complete-supply-chain-account
Accepted: first alpha scope from CAR13; revisit before stable or consumer demand

GitHub build provenance binds source and executable subjects. Complete SBOM, transparency-log and
cross-ecosystem signing coverage remain outside the first alpha claim.
