# Verification: release-artifact-contract

## Package candidates

Run the native pack command for every selected Rust, NuGet and npm package. Inspect the produced
archive metadata and complete file list. The evidence is universal over the selected package
catalog and component-scoped because the claim depends on the real ecosystem packers and archive
formats.

## Catalog drift

Exercise synthetic catalogs and manifests that change one version, identity, license, support
classification or platform entry at a time. Every mismatch must fail with the artifact and field
named. Reordering set-like catalog entries must not change the result.

## Image metadata boundary

C2 validates declared GHCR identities and platforms and builds both Dockerfiles locally. Multi-
architecture image construction, digest verification and provenance attestations remain evidence
for release orchestration, because no public image exists during this change.

## Clean-checkout condition

Run package qualification from a clean temporary checkout so ignored or locally generated files
cannot satisfy a required packed-content assertion. The complete repository gate then proves that
metadata alignment did not break ordinary development builds and tests.
