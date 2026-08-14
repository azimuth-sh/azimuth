# Design: framework/release-artifacts

## Requirement: first-alpha-contract
Mechanism: release-artifact-qualification
Enforcement: guard
Binding: release-artifact-contract

The release catalog is the only repository-owned declaration of the complete first-alpha artifact
set. Native manifests retain ecosystem-required copies, and qualification rejects any difference
before an artifact can enter later publication orchestration. Removing the guard would make
registry metadata and support claims independent again.

## Residue

The guard qualifies local package candidates. It does not establish registry ownership, public
retrievability, native cross-compilation, multi-platform image indexes or release provenance.
