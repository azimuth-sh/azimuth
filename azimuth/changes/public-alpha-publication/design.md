# Design: public-alpha-publication

## Retained candidates are the publication input

The publication workflow accepts a rehearsal run id and an annotated tag. It downloads every
`candidates-*` artifact and the candidate account from that run, then verifies the account before
contacting a writable registry. The account revision must equal the peeled tag revision and the
rehearsal run source revision. Rebuilding in the publication workflow is forbidden because a
successful build would still produce bytes outside the reviewed retained account.

## Reads precede one closed-world plan

Provider adapters normalize public registry responses into the existing planner state. The state
contains every selected key exactly once and distinguishes only absent, exact and conflicting
targets. The operation retrieves all targets before the first write and asks the accepted planner
for the absent set. An adapter error is unknown state and fails closed; it is never mapped to
absence.

## Credential checks precede writes

The workflow verifies all five writable boundaries after read-only state retrieval and before
publishing the first target. The check proves scoped access without reserving a name. Where a
registry offers no non-mutating authorization probe, the credential's presence and authenticated
identity are checked and the first publication remains the unavoidable permission test. That
limitation is recorded rather than represented as stronger evidence.

## Publication is resumable, not transactional

Registry writes remain independent because no cross-registry transaction exists (CAR12). Each
adapter receives the planner-selected keys for its registry and publishes exact retained files.
After any failure, the workflow stops; a rerun starts with fresh public reads and therefore
preserves successful immutable targets while selecting only those still absent.

The GitHub Release is a prerelease under the annotated tag. Its assets are the three native
archives, `SHA256SUMS` and `candidates.json`. GHCR publication copies retained OCI layouts into
versioned multi-platform indexes and attaches GitHub provenance to the resulting index digests.

## Completion is a new public observation

Publication success is insufficient. A final adapter pass retrieves package versions and content
identities, GitHub Release asset digests and GHCR index/platform digests. The accepted completion
checker compares that closed population with the retained account. Its receipt records the tag,
source revision, rehearsal run, publication run, observation time and normalized public state.

## Failure boundaries

- A missing or lightweight tag fails before registry access.
- A rehearsal run from another revision fails before credential checks.
- A missing credential or unauthorized npm scope fails before the first write when the provider
  exposes a non-mutating probe.
- A timeout, rate limit or malformed response is unknown state, not absence.
- A conflict in any registry suppresses the complete publication plan.
- A mid-operation failure leaves already published immutable targets intact and produces no
  completion receipt.
- A public target without the required checksum, platform population or provenance prevents
  completion even when its version string exists.
