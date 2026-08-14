# Outcome: release-artifact-contract

Status: accepted

## Result

The repository now has one machine-readable first-alpha contract for version
`0.1.0-alpha.1` and tag `v0.1.0-alpha.1`. It names five public packages, two public images,
three native targets, both image platforms, five supported surfaces and eleven experimental
source roots. Native Cargo, NuGet, npm and image metadata agree with the contract.

The release qualifier invokes the real Cargo, NuGet and npm packers, reads each candidate's native
metadata and classifies every packed file against the declared allowlist. An independent approved
account prevents a self-consistent catalog and qualifier from accepting unapproved drift. The
qualifier writes stable semantic evidence and the linkage manifest consumed by `azimuth check`.

No implementation command reserves, authenticates to or publishes to a public registry.

## Evidence executed

- Seven release-qualification tests passed. Eight independent mutations covered version, tag,
  license, identities, native targets, image platforms, supported surfaces and experimental roots.
- A clean checkout packed all five candidates without `--allow-dirty`: the Rust crate contained 24
  files, the NuGet packages 7 and 10 files, and the npm packages 3 and 16 files.
- The clean-checkout and working-tree qualification runs produced identical claim fingerprints.
  The clean checkout had no tracked or untracked files after qualification.
- Both assurance Dockerfiles built successfully as Linux ARM64 images. Inspection found
  `0.1.0-alpha.1` and `Apache-2.0` OCI labels on both images.
- The complete repository gate passed, including the Rust tool suite, .NET and TypeScript
  extractors, polyglot conformance, assurance extension conformance, the PostgreSQL-backed service
  component, and the Next.js typecheck and production build.
- The current model reports 7 claims in 2 specs, no holes, no errors and no warnings. All five new
  standard claims carry current `sound` judgments.

## Departures

The initial completion arithmetic said six registry identities while naming seven. The proposal
was corrected to seven before implementation; no selected identity changed.

The first qualifier design compared the catalog only with ecosystem metadata. Audit showed that a
self-consistent edit to both could emit evidence for an unapproved contract, so implementation
added a separate, exact approved account and mutation cases for each contract dimension.

The first evidence payload recorded archive byte hashes and generated NuGet metadata names. A
clean rebuild proved those values non-reproducible and made sound judgments stale. The final payload
records native metadata plus the number of files matched by each allowed pattern. The qualifier
still inspects every actual archive path and fails on a missing, forbidden or undeclared file.

NuGet initially warned that both package readmes were absent. Both packages now include the
canonical repository README, and the allowlists require it.

## Residual decisions

- Reserve and publish the seven registry identities only in a later release-orchestration change.
- Add checksums, provenance, signing and partial-publication recovery before automating publication.
- Cross-compile the three native targets and build multi-architecture image indexes in release CI;
  this change qualifies their declared contract and locally builds only the host image platform.
- Define authentication, secrets and production deployment policy before operating the assurance
  service outside dogfood environments.
- Make compatibility commitments only after evidence from more than one alpha; this alpha promises
  none between prereleases.
- Keep polyglot packages and generic experiments unpublished until a later proposal selects them.

## Measurements

- synchronized release versions: 1;
- selected public identities: 7;
- package candidates: 5, containing 60 files in total;
- image contracts: 2, each declaring 2 platforms;
- native targets: 3;
- supported surfaces: 5;
- experimental source roots: 11;
- release tests: 7, including 8 contract-dimension mutations;
- new standard claims judged sound: 5; and
- final model findings: 0.
