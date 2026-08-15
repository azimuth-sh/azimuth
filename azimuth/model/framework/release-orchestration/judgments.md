# Judgments: framework/release-orchestration

## Claim: ordinary-ci-excludes-release-only-matrix
Verdict: sound
Fingerprint: 8a0bd10d3788a4fe
Judged: 2026-08-15
Judge: Codex

I inspected the sole ordinary workflow command, the root gate's explicit release-image branch, the
workflow account guard and the revision-bound receipt for GitHub run 31860141749. The canonical
`./scripts/check.sh` job completed in 456 seconds without selecting that branch. The transfer change
accepts the historical URL only as one of two exact repository identities; it does not weaken the
workflow, root-gate digest or duration checks. Diagnostic run 31874120337 repeated the current root
path in 429 seconds and failed only after it reached the deliberately stale release receipt. A wrong
workflow with an extra command, an implicit image qualification or a duration at the 45-minute limit
would fail receipt validation or the static account. The successful receipt remains the evidence;
the diagnostic run only confirms that the identity revision did not introduce the release matrix.

## Claim: selected-lanes-are-independent
Verdict: sound
Fingerprint: 7cdcd68c5bcff494
Judged: 2026-08-15
Judge: Codex

I inspected the four-job DAG, both non-fail-fast matrices, the per-lane artifact uploads and the
account's `always()` dependency over all producer jobs. The mutation loop removes each lane and the
static account rejects it. Earlier GitHub run 31859874354 supplied the relevant failure case: its
package lane failed while all three native lanes and both image lanes completed and retained their
artifacts, after which the account failed closed. GitHub run 31883286632 passed the revised
complete DAG. A wrong serial dependency or fail-fast matrix would contradict both the source
account and that observed failure boundary.

## Claim: complete-account-needs-every-lane
Verdict: sound
Fingerprint: 8eacdb862603c77d
Judged: 2026-08-15
Judge: Codex

I inspected catalog-derived subject enumeration, recursive file indexing and the account assembly
loop. The evidence removes and duplicates every one of the ten selected filenames independently
and adds an unexpected file; every mutation fails before an account is returned. GitHub run
31883286632 then downloaded all six lane artifacts and assembled exactly ten subjects. A wrong
assembler accepting one absent, duplicated or extra output is discriminated across the full
catalog population, so the component-universal tag is honest.

## Claim: tag-catalog-and-revision-agree
Verdict: sound
Fingerprint: 1993ae68cca543e8
Judged: 2026-08-15
Judge: Codex

I inspected the catalog tag/version checks, full-commit validation and `git rev-list` comparison
against the annotated tag. The test constructs a real repository and tag, then rejects a different
full revision; independent invalid tag and short-revision cases exercise the other inputs. The
hosted candidate account records tag `v0.1.0-alpha.1` and execution revision
`7f2d78ed982e05c9a316fa18ade8e3592fdaa86c`, which the workflow tagged before assembly. A tag
pointing anywhere else cannot pass this choke point.

Run 31882982629 supplied the adverse case: all producer lanes passed, but the account rejected the
fetched public tag because it named the preceding candidate. The repaired pull-request path forces
only its checkout-local synthetic tag, and run 31883286632 then passed at its merge revision. The
workflow account rejects removal of that PR-only force while owner dispatch retains an existing
fetched tag, so review isolation does not weaken the publication tag boundary.

I also inspected the publication preflight. It requires an annotated tag, reuses the retained
account verifier and independently compares the rehearsal run revision with the account and peeled
tag revision before registry state or credentials can authorize a write. This realization
establishes the same predicate at the public-operation boundary rather than merely transporting the
tag value. The focused wrong-checkout test observes both tag commands and requires the requested
root as their working directory; resolving either command against the agent's current checkout no
longer passes.

The repaired workflow can accept the immutable candidate tag explicitly when orchestration runs
from a later reviewed revision. Candidate authority is unchanged: the account, rehearsal revision
and peeled tag must still agree. The separately recorded publication revision identifies the code
that performed the external operation and cannot substitute for any candidate-side comparison.

## Claim: retained-downloads-have-checksums
Verdict: sound
Fingerprint: 8a344a77410f6c01
Judged: 2026-08-15
Judge: Codex

I inspected the exact filename index, byte-size and streaming SHA-256 account, and the verifier's
comparison with retained files. The mutation ranges over every selected subject and changes its
bytes; each must fail size or digest comparison. I also hashed all ten artifacts downloaded from
GitHub run 31883286632 and matched every digest to `candidates.json`, whose own digest is
`0d58e655f9801b8f8ee90886a2e163c7910215d3c64986d412edf851cfdb5500`. A substituted retained
download therefore cannot retain a passing account.

The publication preflight downloads the cross-run candidates, invokes that verifier and derives an
image registry digest only after the retained OCI archive checksum passes. The public account keeps
the archive checksum separately, so this realization does not substitute a registry manifest
digest for retained-byte identity. The image-provenance job now downloads the complete candidate
population and runs the same verifier before deriving its registry-attestation subject; a partial
or substituted cross-run download therefore fails at that second consumption boundary too.

NuGet.org repository signing changes raw archive bytes after publication. The revised account keeps
the retained archive SHA-256 for provenance and derives a separate path-and-payload digest that
excludes only `.signature.p7s`. The regression changes signature bytes and ZIP order without
changing that digest, then changes a payload and requires a different digest. The public adapter
also requires `dotnet nuget verify --all` to report a repository signature before it can represent
the retained checksum as the published candidate identity.

## Claim: executable-subjects-have-provenance
Verdict: sound
Fingerprint: 377eff7999045d64
Judged: 2026-08-15
Judge: Codex

I inspected each provenance step and queried GitHub's attestation API using the ten retained
SHA-256 digests from exact-candidate run 31901952648. The signed bundle population contains all five
package subjects plus each of the three native archives and two OCI archives; every bundle names
workflow `.github/workflows/release.yml` and execution revision
`49d350b9d3cacc1cfddd8874b97ba67301090960`. A missing or substituted subject would fail the
receipt population or digest lookup.

The repaired image path permits direct provenance when candidate and publication revisions agree,
or a chain when immutable public targets force a later orchestration repair. The chain test requires
retained-archive provenance at the candidate revision and published-digest provenance at the
publication revision; removing either lookup fails. Deterministic OCI inspection already binds the
two digests. This is component evidence for the guard, not operational evidence that the two public
image digests have now been attested. That rollout-dependent limitation remains explicit.

## Claim: packed-packages-install
Verdict: sound
Fingerprint: c5896b4f55115161
Judged: 2026-08-15
Judge: Codex

I inspected all three disposable consumer implementations and the hosted package lane. Cargo
installs the retained crate with `--locked`; NuGet restores both retained packages from an isolated
source and invokes their entry points; npm installs both tarballs into a temporary consumer and
exercises annotation and emitter behavior. Run 31883286632 completed that exact five-package path
using the two `@azimuth-sh` tarballs.
A package that only works through workspace source, omits its executable or exports the wrong API
would fail before the lane artifact is accepted.

## Claim: native-binaries-run
Verdict: sound
Fingerprint: 006fbcdacb2ef8aa
Judged: 2026-08-15
Judge: Codex

I inspected the catalog-derived native matrix, runner mapping, archive construction and extraction
path. Each lane executes the binary from its retained archive and compares `--version` with the
catalog. GitHub run 31883286632 passed on Linux x86-64, macOS ARM64 and Windows x86-64. A wrong
archive name, missing executable bit, incompatible binary or version drift would fail on its
selected runner, so the universal population is the complete three-target catalog set.

## Claim: selected-image-platforms-start
Verdict: sound
Fingerprint: 300be7fa446593db
Judged: 2026-08-15
Judge: Codex

I inspected catalog-derived image matrices, recursive OCI platform inspection, per-platform
`skopeo` import and both startup oracles. The API uses a real PostgreSQL container and `/health`;
the web candidate must answer HTTP through its declared port. Run 31883286632 exercised AMD64 and
ARM64 for both renamed images. Dropping an index manifest, producing a non-starting architecture or
bypassing the declared entry point fails before attestation and upload, so all four selected pairs
are covered.

## Claim: exact-existing-target-is-preserved
Verdict: sound
Fingerprint: af97bb976926bc7e
Judged: 2026-08-15
Judge: Codex

I inspected the planner's complete target map and its identity, digest and image-platform equality
checks. The exact-state evidence supplies all ten subjects and requires an empty publish set plus
ten preserved keys. The same test changes the kind-specific comparison fields and requires a
conflict, so an implementation that simply preserved every existing identity would not pass. The
loop contains no subject-specific exception; the component-universal tag therefore follows the
catalog-derived population.

I also inspected the public adapters and write boundary. They retrieve all selected providers into
one state, preserve omitted keys as absent, keep provider errors distinct from absence and rederive
the supplied plan immediately before writes. No provider adapter can independently bypass the
planner for a target classified exact. The npm adapter also rejects non-HTTPS and non-registry
tarball locations before reading bytes, so remote metadata cannot substitute an unrelated local or
network source while retaining an exact classification.

Write run 31902402263 stopped at npm after a partial release. Read-only run 31902521510 then
retrieved both images, all three native archives and the Rust crate as exact and preserved those six
while selecting four package targets. This is direct operational evidence for a heterogeneous
partial state, not a replacement for the catalog-derived universal evidence.

Later read-only run 31902967972 retrieved both NuGet packages but the former raw-byte adapter called
their provider-added repository signatures conflicts. Direct comparison found every retained path
and payload equal, and the official NuGet verifier accepted both signatures and reported the same
content hashes as the retained candidates. The repaired adapter classified both payload identities
as exact locally; a hosted preflight must still establish the eight-target preservation account.

## Claim: absent-target-is-selected
Verdict: sound
Fingerprint: 992671b2f865e82f
Judged: 2026-08-15
Judge: Codex

I inspected the absent branch in the planner and the mutation loop over the complete ten-subject
account. Each mutation removes exactly one target and requires that target alone in `publish` and
absent from `preserve`. A wrong planner that republishes an exact neighbor, skips one ecosystem or
selects the whole release after a partial failure would fail on the affected iteration. The direct
component evidence ranges over every selected target rather than one representative absence.

The new write path consumes only the planner's `publish` keys and recomputes that plan from the
retained account and observed state before dispatch. The adapter tests deliberately remain ordinary
detector evidence: one selected-write example does not replace the existing universal
catalog-derived absence mutation.

The same read-only run observed both NuGet identities absent two minutes after the failed write
operation. That observation does not prove the preceding push was rejected rather than still
indexing. Selection remains the specified planner result, but recovery must wait for a later public
observation before treating a newly written provider's immediate absence as permission to retry.

## Claim: conflicting-target-fails
Verdict: sound
Fingerprint: 87490fe610a22d01
Judged: 2026-08-15
Judge: Codex

I inspected the single pre-plan conflict accumulator and its uniform loop over every selected key.
The evidence changes the checksum for a package, identity for a native archive and platform set for
an image, requiring each registry kind to raise before returning any plan. These mutations cover
the three distinct comparison shapes; the implementation contains no identity-specific branch
that could exempt another subject. A wrong comparison that ignored content or image platforms
would therefore accept one of the constructed conflicts and fail the oracle.

The provider boundary maps authorization failures, rate limits and malformed responses to errors,
not absence, and separately rejects a conflicting GitHub Release support asset before planning.
Focused fixtures now exercise malformed Cargo, npm, GitHub Release and GHCR responses plus a GHCR
command failure. This realizes fail-closed public observation without overstating those provider
fixtures as new universal Covers evidence.

The NuGet detector additionally rejects an invalid or absent repository signature before planning,
and a non-signature payload mutation yields a conflicting checksum. Excluding `.signature.p7s`
therefore does not turn provider signing into a general content exception.

## Claim: completion-needs-public-retrieval
Verdict: sound
Fingerprint: 2111b66fe1a1dc14
Judged: 2026-08-15
Judge: Codex

I inspected completion as the planner's only success boundary. The test removes every target and
removes provenance from every target independently; both populations must fail, while platform
drift is also rejected for each image. A constant-complete result or a check of only one registry
kind cannot pass. This evidence establishes the component guard over supplied registry state, not
fresh public retrieval. The claim and design now bind concrete package, GitHub Release and GHCR
adapters plus a post-provenance completion job. No public completion receipt exists, so the
accepted rollout-dependent residual remains. This verdict does not authorize calling the alpha
publication complete before real public reads discharge it.

The partial operation and follow-up observations do not discharge that residual. Eight targets are
now publicly retrievable with matching content identities, but the two npm packages remain absent
and both images still lack published-digest provenance. Targeted rerun attempt 2 of run 31902402263
preserved the candidate SHA but GitHub skipped the provenance job again because its failed
dependency remained failed; workflow text and a skipped job cannot satisfy the public oracle.
