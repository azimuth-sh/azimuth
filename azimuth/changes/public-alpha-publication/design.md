# Design: public-alpha-publication

## Independent identity precedes candidate retention

The release repository, npm scope, package source metadata, product homepage and GHCR owner change
as one catalog revision. Package manifests and image labels remain ecosystem-required copies whose
values are checked against that catalog. Historical receipts remain unchanged because they record
executions before the repository transfer; none can qualify candidates after the identity change.

## Retained candidates are the publication input

The publication workflow accepts a rehearsal run id and an annotated tag. It downloads every
`candidates-*` artifact and the candidate account from that run, then verifies the account before
contacting a writable registry. The account revision must equal the peeled tag revision and the
rehearsal run source revision. Rebuilding in the publication workflow is forbidden because a
successful build would still produce bytes outside the reviewed retained account.

The workflow may run from a later reviewed repair revision when the immutable candidate tag is
supplied explicitly. The candidate account and rehearsal must still name the peeled tag revision;
the completion receipt records the separate publication revision. Image provenance is direct when
both revisions are equal. Otherwise, completion requires candidate-revision provenance for the
retained OCI archive, deterministic equality between that archive's index and the public digest,
and publication-revision provenance for the public digest.

Pull-request rehearsals replace the catalog tag only in their local checkout, because an existing
public tag may name the preceding candidate while a repair is under review. Owner-dispatched
rehearsals do not replace an existing tag, so the publication input remains bound to the operator's
annotated public ref.

## Reads precede one closed-world plan *(revised)*

Provider adapters normalize public registry responses into the existing planner state. The state
contains every selected key exactly once and distinguishes absent, exact and conflicting immutable
targets. npm state also records distribution tags because those names are mutable metadata rather
than tarball identity. The operation retrieves all targets before the first write and asks the
accepted planner for the absent set plus npm tag normalizations. An adapter error is unknown state
and fails closed; it is never mapped to absence.

## Credential checks precede writes *(revised)*

The workflow verifies credential presence for all five writable boundaries after read-only state
retrieval and before publishing the first target. It performs provider-supported non-mutating
identity checks: npm must resolve the token identity as an `@azimuth-sh` owner or administrator.
A crates.io token restricted to `publish-new` cannot call the legacy `/me` endpoint, NuGet exposes
no push authorization probe, and a repository read does not prove the job-scoped GitHub token's
Release or GHCR write rights. Those values remain unknown until their first writes. The preflight
records each limitation instead of requiring broader credentials or representing presence as
authenticated access.

## Publication is resumable, not transactional

Registry writes remain independent because no cross-registry transaction exists (CAR12). Each
adapter receives the planner-selected keys for its registry and publishes exact retained files.
The npm adapter derives a non-`latest` distribution tag from a prerelease channel such as `alpha`
or `rc`; stable versions use `latest`. npm rejects prerelease publication without that explicit
channel, but the first public version of a package may still acquire `latest`. After publishing
selected tarballs, the adapter therefore ensures the derived channel points to the version and
removes `latest` only when it points to that prerelease. An exact tarball with tag drift remains in
the immutable preserve set and enters a separate normalization set; it is never republished.
NuGet.org adds a repository signature during ingestion, so its downloadable archive is not
byte-identical to the retained unsigned candidate. The adapter requires the official NuGet
signature verifier to accept that archive, compares every non-signature path and payload through a
signature-independent digest, and records both the public archive digest and retained provenance
digest. Treating the added `.signature.p7s` entry as a conflict would make every successful
NuGet.org publication irrecoverable.
After any failure, the workflow stops; a rerun starts with fresh public reads and therefore
preserves successful immutable targets while selecting only those still absent.

The GitHub Release is a prerelease under the annotated tag. Its assets are the three native
archives, `SHA256SUMS` and `candidates.json`. GHCR publication copies retained OCI layouts into
versioned multi-platform indexes and attaches GitHub provenance to the resulting index digests.

## Completion is a new public observation

Publication success is insufficient. A final adapter pass retrieves package versions and content
identities, npm distribution tags, GitHub Release asset digests and GHCR index/platform digests.
The accepted completion checker compares that closed population with the retained account and
rejects a prerelease whose channel does not select it or whose `latest` tag does. Its receipt
records the tag, source revision, rehearsal run, publication run, observation time and normalized
public state. When repaired orchestration executes after the candidate tag, it also records the
distinct publication revision.

## Failure boundaries

- A missing or lightweight tag fails before registry access.
- A rehearsal run from another revision fails before credential checks.
- An explicitly supplied candidate tag that does not match the retained account fails before
  registry access.
- A missing credential or unauthorized `@azimuth-sh` scope fails before the first write. An
  unprobeable write authorization fails at its first provider write and leaves later reruns to the
  closed-world planner.
- A timeout, rate limit or malformed response is unknown state, not absence.
- A conflict in any registry suppresses the complete publication plan.
- A mid-operation failure leaves already published immutable targets intact and produces no
  completion receipt.
- A provider may accept a write before its public read surface exposes it. Recovery waits for that
  state to settle and repeats the no-write preflight before selecting another write.
- An invalid NuGet repository signature or a changed non-signature payload fails before the
  settled package can be preserved.
- An exact npm tarball with distribution-tag drift is preserved as immutable content but prevents
  completion until the bounded metadata normalization is observed publicly.
- A public target without the required checksum, platform population or provenance prevents
  completion even when its version string exists.
