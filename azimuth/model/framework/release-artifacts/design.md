# Design: framework/release-artifacts

## Requirement: first-alpha-contract
Mechanism: release-artifact-qualification
Enforcement: guard
Binding: release-artifact-contract

The release catalog is the only repository-owned declaration of the complete first-alpha artifact
set. Native manifests retain ecosystem-required copies, and qualification rejects any difference
before an artifact can enter later publication orchestration. Removing the guard would make
registry metadata and support claims independent again.

## Requirement: experimental-source-isolation
Mechanism: experimental-isolation-gate
Enforcement: guard
Binding: experimental-isolation-gate

The gate derives its population from the release catalog, resolves experiment directories to
their root invocations and resolves package or extractor roots through direct commands or manifest
paths. It also checks the single hosted workflow, every tracked executable input in that
population and every retained-domain reference. Covering evidence is emitted only when the root
sequence runs every experiment gate before release qualification. Removing the guard would allow
the release classification, executable gates and provenance boundary to drift independently.

## Residue

The release guard qualifies local package candidates. It does not establish registry ownership,
public retrievability, native cross-compilation, multi-platform image indexes or release
provenance. The isolation guard validates the hosted workflow definition locally; acceptance still
waits for a successful execution attributable to the exact implementation revision.
