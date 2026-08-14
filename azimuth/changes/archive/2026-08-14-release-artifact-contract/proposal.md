# Change: release-artifact-contract

Status: accepted and complete

Exploration: canonical-alpha-release
Carries decisions: CAR2, CAR3, CAR4, CAR5, CAR8, CAR9, CAR10

## Problem

The selected first-alpha artifacts do not yet share a release contract. Rust declares `0.1.0`,
the TypeScript packages declare `0.0.1`, the TypeScript emitter is private and uses the obsolete
`@azimuth/emit-ts` identity, and the .NET emitter lacks the agreed package identity and version.
No machine-readable account currently distinguishes supported alpha artifacts from experimental
source or states which packed contents and platforms are qualified.

Publishing from this state would allow registries and workflows to disagree about version,
identity, license, support or platform coverage. Registry versions are immutable, so discovering
that disagreement only after publication would require another alpha rather than repairing the
same release.

## Outcome

One repository-owned artifact catalog declares `0.1.0-alpha.1`, source tag
`v0.1.0-alpha.1`, public registry identities, Apache-2.0 licensing, supported versus experimental
surfaces and the qualified platform matrix. Ecosystem manifests agree with that catalog.

Deterministic checks build and pack every selected package without publishing it, inspect the
packed metadata and allowlisted contents, and reject version, identity, license, support or
platform drift. The catalog becomes an input to later release orchestration; it does not publish
or attest artifacts itself.

## Scope

In scope:

- a machine-readable, repository-owned catalog for the synchronized first-alpha contract;
- crates.io identity `azimuth` for the Rust CLI and library;
- NuGet identities `Azimuth.Annotations` and `Azimuth.Emit`;
- npm identities `@azimuth/annotations` and `@azimuth/emit`;
- GHCR image identities `ghcr.io/drim-dev/azimuth-assurance-api` and
  `ghcr.io/drim-dev/azimuth-assurance-web`;
- native CLI targets Linux x64, macOS ARM64 and Windows x64;
- assurance image platforms Linux AMD64 and ARM64;
- Apache-2.0, repository and description metadata for public packages;
- explicit supported-alpha and experimental-source classifications; and
- local build, pack and packed-content validation for every package ecosystem.

Out of scope:

- reserving or publishing any registry identity;
- release workflows, checksums, attestations, signing or partial-publication recovery;
- cross-compiling the native binaries or building multi-architecture image indexes;
- private assurance deployment hardening;
- publishing polyglot implementations or generic experiments; and
- compatibility promises between alpha releases.

## Affected claims

Add `framework/release-artifacts#one-source-version`,
`framework/release-artifacts#registry-identities-match-contract`,
`framework/release-artifacts#packed-contents-are-bounded-and-licensed`,
`framework/release-artifacts#support-and-platforms-are-explicit` and
`framework/release-artifacts#experimental-source-is-not-published` at standard criticality.

Publishing incorrect immutable metadata would block the public dogfood path and require a new
prerelease version, but it does not corrupt durable user data or affect an existing consumer.
Standard criticality therefore matches the consequence.

## Completion conditions

- One catalog declares version `0.1.0-alpha.1`, tag `v0.1.0-alpha.1`, the seven registry identities,
  three native targets, two image platforms and Apache-2.0.
- Rust, NuGet, npm and assurance-image metadata derive from or are checked against that catalog;
  no selected artifact declares a conflicting version, identity or license.
- `cargo package`, both `dotnet pack` operations and both `npm pack` operations succeed from a
  clean checkout without registry publication.
- A deterministic checker inspects every packed manifest and file list, rejects undeclared files,
  and proves that build outputs, fixtures, tests, secrets and unrelated repository content are not
  shipped.
- The Rust crate contains both the CLI and reusable library; the integration packages contain the
  public annotations or extractor entry points they advertise.
- Documentation names supported alpha surfaces and qualified platforms, labels polyglot and
  generic experiment trees as experimental source, and makes no unsupported compatibility claim.
- Synthetic drift cases fail for each catalog dimension, and the complete repository check passes.
- No command in this change publishes, reserves or authenticates to a public registry.
