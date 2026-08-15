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

Rehearsal uses a temporary annotated tag over the tested revision and rejects independent changes
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
Residual: GHCR image-digest provenance is absent until the images are published
Accepted: the rehearsal attests retained OCI archives; discharge during alpha publication

The hosted rehearsal submits every downloadable subject to GitHub's provenance action and validates
the workflow's exact subject population. Image publication provenance remains rollout-dependent
until GHCR digests exist.

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
Residual: registry state is synthetic rather than retrieved from public targets
Accepted: no alpha target exists; bind registry adapters and discharge during alpha publication

The completion checker ranges over the complete public population and rejects each missing or
mismatched package, native archive, image index, checksum, provenance subject or platform. A passed
receipt remains rollout-dependent until the release operation retrieves real public targets.

## Residual: registry-credentials-are-not-rehearsed
Accepted: offline orchestration change; revisit before the publication operation

No credential is used during rehearsal. Publication cannot begin until every registry identity and
permission is verified through its bounded release environment.

## Residual: public-completion-is-rollout-dependent
Accepted: no alpha target is published; discharge during `v0.1.0-alpha.1` publication

Dry registry fixtures can establish planner semantics but cannot show that real public targets are
retrievable. The release remains incomplete until the publication operation supplies that evidence.

## Residual: provenance-is-not-a-complete-supply-chain-account
Accepted: first alpha scope from CAR13; revisit before stable or consumer demand

GitHub build provenance binds source and executable subjects. Complete SBOM, transparency-log and
cross-ecosystem signing coverage remain outside the first alpha claim.
